/*
 * gk-init-museum: PID 1 of every booted cell from 1.x to 2.4 (spec 07.4 and 07.5).
 *
 * It is the museum sibling of init.c, for i386 only, and it uses only system calls that 1.0 already had, made with int 0x80. There is no C library, no /proc and no tmpfs to lean on. The kernel passes the words of its command line that hold an = sign to init as environment, which is where gk.suite and gk.kernel come from, since /proc/cmdline came later.
 *
 * It lives as /sbin/init on a small Minix root image that gk writes, and runs the museum smoke suite: fork and wait, exec of itself in a child mode, a pipe round trip, a file written and read back on the root file system, sync, and uname against gk.kernel. Every line it prints starts with GK-, as in init.c. Kernels before 2.1.25 have no serial console, so the rig reads the VGA text screen instead, and init clears the screen before its first line so that the capture is unambiguous.
 *
 * At the end it waits three seconds, so the last screen is captured, and then restarts the machine, which ends QEMU because the rig runs it with -no-reboot. These kernels cannot power off.
 *
 * Run as /sbin/init child by itself, it exits with 42, which is what the exec check looks for.
 */

typedef unsigned long ulong;
typedef long slong;

#define SYS_exit 1
#define SYS_fork 2
#define SYS_read 3
#define SYS_write 4
#define SYS_open 5
#define SYS_close 6
#define SYS_waitpid 7
#define SYS_unlink 10
#define SYS_execve 11
#define SYS_time 13
#define SYS_lseek 19
#define SYS_getpid 20
#define SYS_sync 36
#define SYS_pipe 42
#define SYS_ioctl 54
#define SYS_reboot 88
#define SYS_olduname 109
#define SYS_uname 122

static slong sys3(slong n, slong a, slong b, slong c)
{
	slong ret;
	__asm__ volatile("push %%ebx\n\tmov %2, %%ebx\n\tint $0x80\n\tpop %%ebx"
			 : "=a"(ret)
			 : "a"(n), "r"(a), "c"(b), "d"(c)
			 : "memory");
	return ret;
}

__asm__(".text\n.global _start\n_start:\n"
	"xor %ebp, %ebp\n"
	"mov %esp, %eax\n"
	"and $-16, %esp\n"
	"sub $12, %esp\n"
	"push %eax\n"
	"call cstart\n"
	"hlt\n");

#define sys0(n) sys3(n, 0, 0, 0)
#define sys1(n, a) sys3(n, (slong)(a), 0, 0)
#define sys2(n, a, b) sys3(n, (slong)(a), (slong)(b), 0)

#define O_RDWR 2
#define O_CREAT 0100
#define O_TRUNC 01000

void *memset(void *d, int c, ulong n)
{
	unsigned char *p = d;
	while (n--)
		*p++ = (unsigned char)c;
	return d;
}

void *memcpy(void *d, const void *s, ulong n)
{
	unsigned char *p = d;
	const unsigned char *q = s;
	while (n--)
		*p++ = *q++;
	return d;
}

int memcmp(const void *a, const void *b, ulong n)
{
	const unsigned char *p = a, *q = b;
	for (; n; n--, p++, q++)
		if (*p != *q)
			return *p - *q;
	return 0;
}

static ulong len(const char *s)
{
	ulong n = 0;
	while (s[n])
		n++;
	return n;
}

static int same(const char *a, const char *b)
{
	while (*a && *a == *b)
		a++, b++;
	return *a == *b;
}

static int starts(const char *s, const char *prefix)
{
	while (*prefix)
		if (*s++ != *prefix++)
			return 0;
	return 1;
}

static slong sys_write(int fd, const void *buf, ulong n) { return sys3(SYS_write, fd, (slong)buf, (slong)n); }
static slong sys_read(int fd, void *buf, ulong n) { return sys3(SYS_read, fd, (slong)buf, (slong)n); }
static slong sys_open(const char *path, int flags, int mode) { return sys3(SYS_open, (slong)path, flags, mode); }
static slong sys_close(int fd) { return sys1(SYS_close, fd); }
static slong sys_fork(void) { return sys0(SYS_fork); }
static slong sys_waitpid(slong pid, int *status) { return sys3(SYS_waitpid, pid, (slong)status, 0); }
static slong sys_pipe(int fds[2]) { return sys1(SYS_pipe, fds); }
static slong sys_unlink(const char *path) { return sys1(SYS_unlink, path); }
static slong sys_lseek(int fd, slong at) { return sys3(SYS_lseek, fd, at, 0); }
static slong sys_time(void) { return sys1(SYS_time, 0); }
static slong sys_execve(const char *path, char *const argv[], char *const envp[]) { return sys3(SYS_execve, (slong)path, (slong)argv, (slong)envp); }

static __attribute__((noreturn)) void sys_exit(int code)
{
	for (;;)
		sys1(SYS_exit, code);
}

/* Output, one line at a time, so that lines from the kernel do not land in the middle of ours. */

static char line[256];
static ulong line_len;

static void put(const char *s)
{
	while (*s && line_len < sizeof line - 1)
		line[line_len++] = *s++;
}

static void end_line(void)
{
	line[line_len++] = '\n';
	sys_write(1, line, line_len);
	line_len = 0;
}

static int failures;

static void check(const char *name, int ok)
{
	put("GK-CHECK ");
	put(name);
	put(ok ? " pass" : " fail");
	end_line();
	if (!ok)
		failures++;
}

static char **environ;

static const char *param(const char *key)
{
	ulong k = len(key);
	char **e;
	for (e = environ; e && *e; e++)
		if (starts(*e, key) && (*e)[k] == '=')
			return *e + k + 1;
	return 0;
}

/* The release is the third field in both layouts. new_utsname has 65 bytes a field and old_utsname has 65 too, so one offset serves both, and 1.0 has the new call already. */
static struct {
	char sysname[65], nodename[65], release[65], version[65], machine[65], domainname[65];
} uts;

static const char *release(void)
{
	if (sys1(SYS_uname, &uts) == 0 || sys1(SYS_olduname, &uts) == 0)
		return uts.release;
	return "unknown";
}

static char *self_path = "/sbin/init";

static int check_fork(void)
{
	int status = 0;
	slong pid = sys_fork();
	if (pid == 0)
		sys_exit(7);
	return pid > 0 && sys_waitpid(pid, &status) == pid && (status & 0x7f) == 0 && ((status >> 8) & 0xff) == 7;
}

static int check_exec(void)
{
	int status = 0;
	slong pid = sys_fork();
	if (pid == 0) {
		char *argv[] = {self_path, "child", 0};
		char *envp[] = {0};
		sys_execve(self_path, argv, envp);
		sys_exit(1);
	}
	return pid > 0 && sys_waitpid(pid, &status) == pid && ((status >> 8) & 0xff) == 42;
}

static int check_pipe(void)
{
	int fds[2], status = 0;
	char buf[8] = {0};
	if (sys_pipe(fds) < 0)
		return 0;
	slong pid = sys_fork();
	if (pid == 0) {
		sys_close(fds[0]);
		sys_exit(sys_write(fds[1], "gk-pipe", 7) == 7 ? 0 : 1);
	}
	sys_close(fds[1]);
	int ok = pid > 0 && sys_read(fds[0], buf, 7) == 7 && memcmp(buf, "gk-pipe", 7) == 0;
	sys_close(fds[0]);
	return ok && sys_waitpid(pid, &status) == pid && status == 0;
}

static int check_rootfs(void)
{
	static char data[4096], back[4096];
	ulong i;
	for (i = 0; i < sizeof data; i++)
		data[i] = (char)(i * 7);
	slong fd = sys_open("/tmp/gk-file", O_RDWR | O_CREAT | O_TRUNC, 0644);
	if (fd < 0)
		return 0;
	int ok = sys_write((int)fd, data, sizeof data) == (slong)sizeof data;
	ok = ok && sys_lseek((int)fd, 0) == 0;
	ok = ok && sys_read((int)fd, back, sizeof back) == (slong)sizeof back;
	ok = ok && memcmp(data, back, sizeof back) == 0;
	sys_close((int)fd);
	return sys_unlink("/tmp/gk-file") == 0 && ok;
}

static int check_sync(void)
{
	return sys0(SYS_sync) == 0;
}

static int check_uname(void)
{
	const char *want = param("gk.kernel");
	return want && starts(release(), want);
}

static void smoke(void)
{
	check("pid-1", sys0(SYS_getpid) == 1);
	check("fork-wait", check_fork());
	check("exec", check_exec());
	check("pipe", check_pipe());
	check("rootfs-file", check_rootfs());
	check("sync", check_sync());
	check("uname", check_uname());
}

#define VT_GETSTATE 0x5603
#define VT_ACTIVATE 0x5606
#define VT_WAITACTIVE 0x5607

/*
 * The consoles of the era scroll by moving the start of the screen through video memory, and the rig reads the screen from the start of that memory. Switching to the second console and back copies the first one to the start and moves the screen there, and the suite prints fewer lines than a screen holds, so it stays put. Only a VGA console does this: on a serial console fd 1 is not a VT.
 */
static void home_screen(void)
{
	unsigned short state[3];
	slong fd;
	if (sys3(SYS_ioctl, 1, VT_GETSTATE, (slong)state) != 0)
		return;
	fd = sys_open("/dev/tty0", O_RDWR, 0);
	if (fd < 0)
		return;
	if (sys3(SYS_ioctl, fd, VT_ACTIVATE, 2) == 0) {
		sys3(SYS_ioctl, fd, VT_WAITACTIVE, 2);
		sys3(SYS_ioctl, fd, VT_ACTIVATE, 1);
		sys3(SYS_ioctl, fd, VT_WAITACTIVE, 1);
	}
	sys_close((int)fd);
}

#define LINUX_REBOOT_MAGIC1 0xfee1dead
#define LINUX_REBOOT_MAGIC2 672274793
#define RB_AUTOBOOT 0x01234567

static __attribute__((noreturn)) void restart(void)
{
	slong start = sys_time();
	sys0(SYS_sync);
	while (sys_time() - start < 3)
		;
	sys3(SYS_reboot, LINUX_REBOOT_MAGIC1, LINUX_REBOOT_MAGIC2, RB_AUTOBOOT);
	for (;;)
		;
}

__attribute__((noreturn, used)) void cstart(slong *sp)
{
	int argc = (int)sp[0];
	char **argv = (char **)(sp + 1);
	const char *suite;
	char name[32];

	if (argc > 1 && same(argv[1], "child"))
		sys_exit(42);
	if (argc > 0 && argv[0][0] == '/')
		self_path = argv[0];
	environ = argv + argc + 1;

	/* Clear the screen and home the cursor, so a VGA capture starts from a blank page. A serial console just prints the escapes. */
	home_screen();
	sys_write(1, "\033[H\033[J", 6);
	put("GK-BOOTED ");
	put(release());
	end_line();

	suite = param("gk.suite");
	if (!suite || len(suite) >= sizeof name)
		suite = "smoke";
	suite = memcpy(name, suite, len(suite) + 1);
	put("GK-BEGIN ");
	put(suite);
	end_line();
	if (same(suite, "smoke"))
		smoke();
	else {
		put("GK-NOTE no suite called ");
		put(suite);
		end_line();
		failures++;
	}
	put("GK-END ");
	put(suite);
	put(failures ? " fail" : " pass");
	end_line();
	restart();
}
