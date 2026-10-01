/*
 * gk-init: PID 1 of every booted cell from 2.6.0 on (spec 07.4 and 07.5).
 *
 * It has no C library. Every system call is made with the platform's own instruction, and only calls that 2.6.0 already had are used, so one binary runs on every kernel of its platform. gk builds it with a pinned GCC bundle and packs it into the initramfs as /init, which is the same file for every column of a row.
 *
 * It mounts what the kernel has, prints GK-BOOTED and the release, runs the suite named by gk.suite on the kernel command line, and powers off. Every line the rig reads starts with GK-. A check prints GK-CHECK <name> pass or fail, and a suite is framed by GK-BEGIN <suite> and GK-END <suite> <status>.
 *
 * Run as /init child by itself, it exits with 42, which is what the exec check looks for.
 */

typedef unsigned long ulong;
typedef long slong;

#if defined(__x86_64__)
#define SYS_read 0
#define SYS_write 1
#define SYS_open 2
#define SYS_close 3
#define SYS_mmap 9
#define SYS_munmap 11
#define SYS_brk 12
#define SYS_pipe 22
#define SYS_dup2 33
#define SYS_nanosleep 35
#define SYS_getpid 39
#define SYS_fork 57
#define SYS_execve 59
#define SYS_exit 60
#define SYS_wait4 61
#define SYS_kill 62
#define SYS_uname 63
#define SYS_rename 82
#define SYS_mkdir 83
#define SYS_rmdir 84
#define SYS_unlink 87
#define SYS_sync 162
#define SYS_mount 165
#define SYS_reboot 169
#define SYS_clock_gettime 228

static slong sys6(slong n, slong a, slong b, slong c, slong d, slong e, slong f)
{
	slong ret;
	register slong r10 __asm__("r10") = d;
	register slong r8 __asm__("r8") = e;
	register slong r9 __asm__("r9") = f;
	__asm__ volatile("syscall"
			 : "=a"(ret)
			 : "a"(n), "D"(a), "S"(b), "d"(c), "r"(r10), "r"(r8), "r"(r9)
			 : "rcx", "r11", "memory");
	return ret;
}

__asm__(".text\n.global _start\n_start:\n"
	"xor %rbp, %rbp\n"
	"mov %rsp, %rdi\n"
	"and $-16, %rsp\n"
	"call cstart\n"
	"hlt\n");

#elif defined(__i386__)
#define SYS_exit 1
#define SYS_fork 2
#define SYS_read 3
#define SYS_write 4
#define SYS_open 5
#define SYS_close 6
#define SYS_unlink 10
#define SYS_execve 11
#define SYS_getpid 20
#define SYS_mount 21
#define SYS_sync 36
#define SYS_kill 37
#define SYS_rename 38
#define SYS_mkdir 39
#define SYS_rmdir 40
#define SYS_pipe 42
#define SYS_brk 45
#define SYS_dup2 63
#define SYS_reboot 88
#define SYS_old_mmap 90
#define SYS_munmap 91
#define SYS_wait4 114
#define SYS_uname 122
#define SYS_nanosleep 162
#define SYS_clock_gettime 265

/* Five arguments at most, since ebp is the sixth and GCC may want it as the frame pointer. mmap goes through old_mmap, which takes its six in memory. */
static slong sys6(slong n, slong a, slong b, slong c, slong d, slong e, slong f)
{
	slong ret;
	(void)f;
	__asm__ volatile("push %%ebx\n\tmov %2, %%ebx\n\tint $0x80\n\tpop %%ebx"
			 : "=a"(ret)
			 : "a"(n), "r"(a), "c"(b), "d"(c), "S"(d), "D"(e)
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

#elif defined(__aarch64__)
/* The generic table: no fork, open, pipe, mkdir or the like, only their at and 2 forms. */
#define SYS_dup3 24
#define SYS_mkdirat 34
#define SYS_unlinkat 35
#define SYS_renameat 38
#define SYS_mount 40
#define SYS_openat 56
#define SYS_close 57
#define SYS_pipe2 59
#define SYS_read 63
#define SYS_write 64
#define SYS_sync 81
#define SYS_exit 93
#define SYS_nanosleep 101
#define SYS_clock_gettime 113
#define SYS_kill 129
#define SYS_reboot 142
#define SYS_uname 160
#define SYS_getpid 172
#define SYS_brk 214
#define SYS_munmap 215
#define SYS_clone 220
#define SYS_execve 221
#define SYS_mmap 222
#define SYS_wait4 260

static slong sys6(slong n, slong a, slong b, slong c, slong d, slong e, slong f)
{
	register slong x8 __asm__("x8") = n;
	register slong x0 __asm__("x0") = a;
	register slong x1 __asm__("x1") = b;
	register slong x2 __asm__("x2") = c;
	register slong x3 __asm__("x3") = d;
	register slong x4 __asm__("x4") = e;
	register slong x5 __asm__("x5") = f;
	__asm__ volatile("svc #0"
			 : "+r"(x0)
			 : "r"(x8), "r"(x1), "r"(x2), "r"(x3), "r"(x4), "r"(x5)
			 : "memory");
	return x0;
}

__asm__(".text\n.global _start\n_start:\n"
	"mov x29, #0\n"
	"mov x0, sp\n"
	"bl cstart\n"
	"1: b 1b\n");

#define AT_FDCWD (-100)
#define AT_REMOVEDIR 0x200
#else
#error "gk-init has no system calls for this platform yet"
#endif

#define sys0(n) sys6(n, 0, 0, 0, 0, 0, 0)
#define sys1(n, a) sys6(n, (slong)(a), 0, 0, 0, 0, 0)
#define sys2(n, a, b) sys6(n, (slong)(a), (slong)(b), 0, 0, 0, 0)
#define sys3(n, a, b, c) sys6(n, (slong)(a), (slong)(b), (slong)(c), 0, 0, 0)
#define sys4(n, a, b, c, d) sys6(n, (slong)(a), (slong)(b), (slong)(c), (slong)(d), 0, 0)
#define sys5(n, a, b, c, d, e) sys6(n, (slong)(a), (slong)(b), (slong)(c), (slong)(d), (slong)(e), 0)

#define O_RDONLY 0
#define O_WRONLY 1
#define O_RDWR 2
#define O_CREAT 0100
#define O_TRUNC 01000
#define PROT_READ 1
#define PROT_WRITE 2
#define MAP_PRIVATE 2
#define MAP_ANONYMOUS 0x20
#define SIGTERM 15
#define SIGCHLD 17
#define CLOCK_MONOTONIC 1

/* GCC may call these for struct copies and zeroing even without a library. */
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

/* System calls by what they do, so the checks read the same on every platform. */

static slong sys_write(int fd, const void *buf, ulong n) { return sys3(SYS_write, fd, buf, n); }
static slong sys_read(int fd, void *buf, ulong n) { return sys3(SYS_read, fd, buf, n); }
static slong sys_close(int fd) { return sys1(SYS_close, fd); }
static slong sys_getpid(void) { return sys0(SYS_getpid); }
static slong sys_kill(slong pid, int sig) { return sys2(SYS_kill, pid, sig); }
static slong sys_wait4(slong pid, int *status) { return sys4(SYS_wait4, pid, status, 0, 0); }
static slong sys_uname(void *buf) { return sys1(SYS_uname, buf); }
static slong sys_brk(void *p) { return sys1(SYS_brk, p); }
static slong sys_munmap(void *p, ulong n) { return sys2(SYS_munmap, p, n); }
static slong sys_nanosleep(const void *req) { return sys2(SYS_nanosleep, req, 0); }
static slong sys_clock_gettime(int clock, void *ts) { return sys2(SYS_clock_gettime, clock, ts); }
static slong sys_mount(const char *src, const char *dst, const char *type) { return sys5(SYS_mount, src, dst, type, 0, 0); }
static slong sys_execve(const char *path, char *const argv[], char *const envp[]) { return sys3(SYS_execve, path, argv, envp); }

static __attribute__((noreturn)) void sys_exit(int code)
{
	for (;;)
		sys1(SYS_exit, code);
}

#if defined(__aarch64__)
static slong sys_open(const char *path, int flags, int mode) { return sys4(SYS_openat, AT_FDCWD, path, flags, mode); }
static slong sys_fork(void) { return sys5(SYS_clone, SIGCHLD, 0, 0, 0, 0); }
static slong sys_pipe(int fds[2]) { return sys2(SYS_pipe2, fds, 0); }
static slong sys_dup2(int a, int b) { return sys3(SYS_dup3, a, b, 0); }
static slong sys_mkdir(const char *path) { return sys3(SYS_mkdirat, AT_FDCWD, path, 0755); }
static slong sys_rmdir(const char *path) { return sys3(SYS_unlinkat, AT_FDCWD, path, AT_REMOVEDIR); }
static slong sys_unlink(const char *path) { return sys3(SYS_unlinkat, AT_FDCWD, path, 0); }
static slong sys_rename(const char *a, const char *b) { return sys4(SYS_renameat, AT_FDCWD, a, AT_FDCWD, b); }
#else
static slong sys_open(const char *path, int flags, int mode) { return sys3(SYS_open, path, flags, mode); }
static slong sys_fork(void) { return sys0(SYS_fork); }
static slong sys_pipe(int fds[2]) { return sys1(SYS_pipe, fds); }
static slong sys_dup2(int a, int b) { return sys2(SYS_dup2, a, b); }
static slong sys_mkdir(const char *path) { return sys2(SYS_mkdir, path, 0755); }
static slong sys_rmdir(const char *path) { return sys1(SYS_rmdir, path); }
static slong sys_unlink(const char *path) { return sys1(SYS_unlink, path); }
static slong sys_rename(const char *a, const char *b) { return sys2(SYS_rename, a, b); }
#endif

static void *sys_mmap(ulong n)
{
#if defined(__i386__)
	ulong args[6] = {0, n, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, (ulong)-1, 0};
	return (void *)sys1(SYS_old_mmap, args);
#else
	return (void *)sys6(SYS_mmap, 0, (slong)n, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
#endif
}

static int mapped(void *p)
{
	return (ulong)p < (ulong)-4096;
}

/* Output, one line at a time, so that lines from the kernel do not land in the middle of ours. */

static char line[512];
static ulong line_len;

static void put(const char *s)
{
	while (*s && line_len < sizeof line - 1)
		line[line_len++] = *s++;
}

static void put_num(ulong n)
{
	char buf[24];
	int i = sizeof buf;
	buf[--i] = 0;
	do
		buf[--i] = (char)('0' + n % 10);
	while ((n /= 10) && i);
	put(buf + i);
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

/* The kernel command line. Words with a dot are module parameters to the kernel, so they never reach init as arguments and have to be read from /proc. */

static char cmdline[4096];

static const char *param(const char *key)
{
	static char value[64];
	ulong k = len(key);
	const char *p = cmdline;
	while (*p) {
		while (*p == ' ' || *p == '\n')
			p++;
		if (starts(p, key) && p[k] == '=') {
			ulong i = 0;
			p += k + 1;
			while (*p && *p != ' ' && *p != '\n' && i < sizeof value - 1)
				value[i++] = *p++;
			value[i] = 0;
			return value;
		}
		while (*p && *p != ' ' && *p != '\n')
			p++;
	}
	return 0;
}

static ulong number(const char *s)
{
	ulong n = 0;
	while (s && *s >= '0' && *s <= '9')
		n = n * 10 + (ulong)(*s++ - '0');
	return n;
}

static slong read_file(const char *path, char *buf, ulong size)
{
	slong fd = sys_open(path, O_RDONLY, 0);
	slong got = 0, n;
	if (fd < 0)
		return fd;
	while (got < (slong)size - 1 && (n = sys_read((int)fd, buf + got, size - 1 - (ulong)got)) > 0)
		got += n;
	buf[got] = 0;
	sys_close((int)fd);
	return got;
}

struct utsname {
	char sysname[65], nodename[65], release[65], version[65], machine[65], domainname[65];
};

struct timespec {
	slong sec, nsec;
};

static char *self_path = "/init";
static char suite_name[64];

/* The checks. Each one is a small function that says whether the kernel did what it should. */

static int check_fork(void)
{
	int status = 0;
	slong pid = sys_fork();
	if (pid == 0)
		sys_exit(7);
	return pid > 0 && sys_wait4(pid, &status) == pid && (status & 0x7f) == 0 && ((status >> 8) & 0xff) == 7;
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
	return pid > 0 && sys_wait4(pid, &status) == pid && ((status >> 8) & 0xff) == 42;
}

static int check_pipe(void)
{
	int fds[2];
	char buf[8] = {0};
	if (sys_pipe(fds) < 0)
		return 0;
	int ok = sys_write(fds[1], "gk-pipe", 7) == 7 && sys_read(fds[0], buf, 7) == 7 && memcmp(buf, "gk-pipe", 7) == 0;
	sys_close(fds[0]);
	sys_close(fds[1]);
	return ok;
}

static int check_pipe_child(void)
{
	int fds[2], status = 0;
	char buf[8] = {0};
	if (sys_pipe(fds) < 0)
		return 0;
	slong pid = sys_fork();
	if (pid == 0) {
		sys_close(fds[0]);
		sys_exit(sys_write(fds[1], "child", 5) == 5 ? 0 : 1);
	}
	sys_close(fds[1]);
	int ok = pid > 0 && sys_read(fds[0], buf, 5) == 5 && memcmp(buf, "child", 5) == 0;
	sys_close(fds[0]);
	return ok && sys_wait4(pid, &status) == pid && status == 0;
}

static int check_signal(void)
{
	int status = 0;
	slong pid = sys_fork();
	if (pid == 0) {
		sys_kill(sys_getpid(), SIGTERM);
		sys_exit(0);
	}
	return pid > 0 && sys_wait4(pid, &status) == pid && (status & 0x7f) == SIGTERM;
}

static int check_mmap(void)
{
	ulong size = 64ul << 20, words = size / sizeof(unsigned int), i;
	unsigned int *p = sys_mmap(size);
	int ok = 1;
	if (!mapped(p))
		return 0;
	for (i = 0; i < words; i++)
		p[i] = (unsigned int)(i * 2654435761u);
	for (i = 0; i < words && ok; i++)
		ok = p[i] == (unsigned int)(i * 2654435761u);
	return sys_munmap(p, size) == 0 && ok;
}

static int check_brk(void)
{
	slong start = sys_brk(0);
	slong end = sys_brk((char *)start + (1 << 20));
	if (end < start + (1 << 20))
		return 0;
	((volatile char *)start)[(1 << 20) - 1] = 1;
	return sys_brk((void *)start) == start;
}

static int check_tmpfs(void)
{
	static char data[8192], back[8192];
	ulong i;
	for (i = 0; i < sizeof data; i++)
		data[i] = (char)(i * 7);
	slong fd = sys_open("/tmp/gk-file", O_RDWR | O_CREAT | O_TRUNC, 0644);
	if (fd < 0)
		return 0;
	int ok = sys_write((int)fd, data, sizeof data) == (slong)sizeof data;
	sys_close((int)fd);
	ok = ok && read_file("/tmp/gk-file", back, sizeof back) == (slong)sizeof back - 1;
	ok = ok && memcmp(data, back, sizeof back - 1) == 0;
	return sys_unlink("/tmp/gk-file") == 0 && ok;
}

static int check_dirs(void)
{
	return sys_mkdir("/tmp/gk-a") == 0 && sys_rename("/tmp/gk-a", "/tmp/gk-b") == 0 &&
	       sys_rmdir("/tmp/gk-a") < 0 && sys_rmdir("/tmp/gk-b") == 0;
}

static int check_dup(void)
{
	slong fd = sys_open("/proc/self/maps", O_RDONLY, 0);
	char buf[16];
	if (fd < 0)
		return 0;
	int ok = sys_dup2((int)fd, 10) == 10 && sys_read(10, buf, sizeof buf) > 0;
	sys_close(10);
	sys_close((int)fd);
	return ok;
}

static int check_proc_maps(void)
{
	static char buf[4096];
	return read_file("/proc/self/maps", buf, sizeof buf) > 0;
}

static int check_devnull(void)
{
	slong fd = sys_open("/dev/null", O_WRONLY, 0);
	if (fd < 0)
		return 0;
	int ok = sys_write((int)fd, "x", 1) == 1;
	sys_close((int)fd);
	return ok;
}

static int check_sysfs(void)
{
	slong fd = sys_open("/sys/kernel", O_RDONLY, 0);
	if (fd < 0)
		return 0;
	sys_close((int)fd);
	return 1;
}

static int check_clock(void)
{
	struct timespec a, b, nap = {0, 20 * 1000 * 1000};
	if (sys_clock_gettime(CLOCK_MONOTONIC, &a) < 0 || sys_nanosleep(&nap) < 0 ||
	    sys_clock_gettime(CLOCK_MONOTONIC, &b) < 0)
		return 0;
	/* In whole milliseconds, which fit in 32 bits for any sane nap and need no division helper from libgcc. */
	slong ms = (b.sec - a.sec) * 1000 + (b.nsec - a.nsec) / 1000000;
	return ms >= 19;
}

static int check_pid(void)
{
	return sys_getpid() == 1;
}

static int check_many_children(void)
{
	int status, n;
	for (n = 0; n < 100; n++) {
		slong pid = sys_fork();
		if (pid == 0)
			sys_exit(n & 0x7f);
		if (pid < 0 || sys_wait4(pid, &status) != pid || ((status >> 8) & 0xff) != (n & 0x7f))
			return 0;
	}
	return 1;
}

static int check_cpus(void)
{
	static char buf[65536];
	const char *want = param("gk.cpus");
	ulong seen = 0;
	const char *p = buf;
	if (!want || read_file("/proc/cpuinfo", buf, sizeof buf) <= 0)
		return 0;
	for (; *p; p++)
		if ((p == buf || p[-1] == '\n') && starts(p, "processor"))
			seen++;
	return seen == number(want);
}

/* The rig passes the kernel version as gk.kernel, and the release the kernel reports has to start with it. */
static int check_uname(void)
{
	struct utsname u;
	const char *want = param("gk.kernel");
	return want && sys_uname(&u) == 0 && starts(u.release, want);
}

static void smoke(void)
{
	check("pid-1", check_pid());
	check("fork-wait", check_fork());
	check("exec", check_exec());
	check("pipe", check_pipe());
	check("pipe-child", check_pipe_child());
	check("signal", check_signal());
	check("mmap-64m", check_mmap());
	check("brk", check_brk());
	check("proc-maps", check_proc_maps());
	check("dev-null", check_devnull());
	check("sysfs", check_sysfs());
	check("tmpfs-file", check_tmpfs());
	check("tmpfs-dirs", check_dirs());
	check("dup2", check_dup());
	check("clock", check_clock());
	check("uname", check_uname());
	check("fork-100", check_many_children());
	check("cpus", check_cpus());
}

#define LINUX_REBOOT_MAGIC1 0xfee1dead
#define LINUX_REBOOT_MAGIC2 672274793
#define RB_AUTOBOOT 0x01234567
#define RB_POWER_OFF 0x4321fedc

static __attribute__((noreturn)) void power_off(void)
{
	sys0(SYS_sync);
	sys4(SYS_reboot, LINUX_REBOOT_MAGIC1, LINUX_REBOOT_MAGIC2, RB_POWER_OFF, 0);
	/* No power off on this machine: a restart ends QEMU too, because the rig runs it with -no-reboot. */
	sys4(SYS_reboot, LINUX_REBOOT_MAGIC1, LINUX_REBOOT_MAGIC2, RB_AUTOBOOT, 0);
	for (;;)
		;
}

__attribute__((noreturn, used)) void cstart(slong *sp)
{
	int argc = (int)sp[0];
	char **argv = (char **)(sp + 1);
	struct utsname u;
	const char *suite;

	if (argc > 1 && same(argv[1], "child"))
		sys_exit(42);
	if (argc > 0 && argv[0][0] == '/')
		self_path = argv[0];

	sys_mount("proc", "/proc", "proc");
	sys_mount("sysfs", "/sys", "sysfs");
	sys_mount("devtmpfs", "/dev", "devtmpfs");
	sys_mount("tmpfs", "/tmp", "tmpfs");
	sys_mount("debugfs", "/sys/kernel/debug", "debugfs");
	if (read_file("/proc/cmdline", cmdline, sizeof cmdline) < 0)
		cmdline[0] = 0;

	put("GK-BOOTED ");
	put(sys_uname(&u) == 0 ? u.release : "unknown");
	end_line();

	/* param returns a buffer the next call reuses, and the checks call it too. */
	suite = param("gk.suite");
	if (suite)
		suite = memcpy(suite_name, suite, len(suite) + 1);
	else
		suite = "smoke";
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
	put(failures ? " fail " : " pass ");
	put_num((ulong)failures);
	end_line();
	power_off();
}
