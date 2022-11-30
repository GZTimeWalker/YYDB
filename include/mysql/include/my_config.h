/* Copyright (c) 2009, 2022, Oracle and/or its affiliates.

 This program is free software; you can redistribute it and/or modify
 it under the terms of the GNU General Public License, version 2.0,
 as published by the Free Software Foundation.

 This program is also distributed with certain software (including
 but not limited to OpenSSL) that is licensed under separate terms,
 as designated in a particular file or component or in included license
 documentation.  The authors of MySQL hereby grant you an additional
 permission to link the program and your derivative works with the
 separately licensed software that they have included with MySQL.

 This program is distributed in the hope that it will be useful,
 but WITHOUT ANY WARRANTY; without even the implied warranty of
 MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 GNU General Public License, version 2.0, for more details.

 You should have received a copy of the GNU General Public License
 along with this program; if not, write to the Free Software
 Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301  USA */

#ifndef MY_CONFIG_H
#define MY_CONFIG_H

/*
 * From configure.cmake, in order of appearance
 */

/* Libraries */
/* #undef HAVE_LIBM */
/* #undef HAVE_LIBNSL */
/* #undef HAVE_LIBSOCKET */
/* #undef HAVE_LIBDL */
/* #undef HAVE_LIBRT */
/* #undef HAVE_LIBWRAP */
/* #undef HAVE_LIBWRAP_PROTOTYPES */

/* Header files */
/* #undef HAVE_ALLOCA_H */
/* #undef HAVE_ARPA_INET_H */
/* #undef HAVE_DLFCN_H */
/* #undef HAVE_EXECINFO_H */
/* #undef HAVE_FPU_CONTROL_H */
/* #undef HAVE_GRP_H */
/* #undef HAVE_LANGINFO_H */
/* #undef HAVE_MALLOC_H */
/* #undef HAVE_NETINET_IN_H */
/* #undef HAVE_POLL_H */
/* #undef HAVE_PWD_H */
/* #undef HAVE_STRINGS_H */
/* #undef HAVE_SYS_CDEFS_H */
/* #undef HAVE_SYS_IOCTL_H */
/* #undef HAVE_SYS_MMAN_H */
/* #undef HAVE_SYS_PRCTL_H */
/* #undef HAVE_SYS_RESOURCE_H */
/* #undef HAVE_SYS_SELECT_H */
/* #undef HAVE_SYS_SOCKET_H */
/* #undef HAVE_TERM_H */
/* #undef HAVE_TERMIOS_H */
/* #undef HAVE_TERMIO_H */
/* #undef HAVE_UNISTD_H */
/* #undef HAVE_SYS_WAIT_H */
/* #undef HAVE_SYS_PARAM_H */
/* #undef HAVE_FNMATCH_H */
/* #undef HAVE_SYS_UN_H */
/* #undef HAVE_SASL_SASL_H */

/* Functions */
/* #undef HAVE_ALIGNED_MALLOC */
/* #undef HAVE_BACKTRACE */
/* #undef HAVE_INDEX */
/* #undef HAVE_CHOWN */
/* #undef HAVE_CUSERID */
/* #undef HAVE_DIRECTIO */
/* #undef HAVE_FTRUNCATE */
/* #undef HAVE_FCHMOD */
/* #undef HAVE_FCNTL */
/* #undef HAVE_FDATASYNC */
/* #undef HAVE_DECL_FDATASYNC */
/* #undef HAVE_FEDISABLEEXCEPT */
/* #undef HAVE_FSYNC */
/* #undef HAVE_GETHRTIME */
/* #undef HAVE_GETPASS */
/* #undef HAVE_GETPASSPHRASE */
/* #undef HAVE_GETPWNAM */
/* #undef HAVE_GETPWUID */
/* #undef HAVE_GETRUSAGE */
/* #undef HAVE_INITGROUPS */
/* #undef HAVE_ISSETUGID */
/* #undef HAVE_GETUID */
/* #undef HAVE_GETEUID */
/* #undef HAVE_GETGID */
/* #undef HAVE_GETEGID */
/* #undef HAVE_LSAN_DO_RECOVERABLE_LEAK_CHECK */
/* #undef HAVE_MADVISE */
/* #undef HAVE_MALLOC_INFO */
/* #undef HAVE_MLOCK */
/* #undef HAVE_MLOCKALL */
/* #undef HAVE_MMAP64 */
/* #undef HAVE_POLL */
/* #undef HAVE_POSIX_FALLOCATE */
/* #undef HAVE_POSIX_MEMALIGN */
/* #undef HAVE_PTHREAD_CONDATTR_SETCLOCK */
/* #undef HAVE_PTHREAD_GETAFFINITY_NP */
/* #undef HAVE_PTHREAD_SIGMASK */
/* #undef HAVE_PTHREAD_SETNAME_NP_LINUX */
/* #undef HAVE_PTHREAD_SETNAME_NP_MACOS */
/* #undef HAVE_SET_THREAD_DESCRIPTION */
/* #undef HAVE_SLEEP */
/* #undef HAVE_STPCPY */
/* #undef HAVE_STPNCPY */
/* #undef HAVE_STRLCPY */
/* #undef HAVE_STRLCAT */
/* #undef HAVE_STRPTIME */
/* #undef HAVE_STRSIGNAL */
/* #undef HAVE_TELL */
/* #undef HAVE_VASPRINTF */
/* #undef HAVE_MEMALIGN */
/* #undef HAVE_NL_LANGINFO */
/* #undef HAVE_HTONLL */
/* #undef HAVE_EPOLL */

/* WL2373 */
/* #undef HAVE_SYS_TIME_H */
/* #undef HAVE_SYS_TIMES_H */
/* #undef HAVE_TIMES */
/* #undef HAVE_GETTIMEOFDAY */

/* Symbols */
/* #undef HAVE_LRAND48 */
/* #undef GWINSZ_IN_SYS_IOCTL */
/* #undef FIONREAD_IN_SYS_IOCTL */
/* #undef FIONREAD_IN_SYS_FILIO */
/* #undef HAVE_MADV_DONTDUMP */
/* #undef HAVE_O_TMPFILE */

/* #undef HAVE_KQUEUE */
/* #undef HAVE_SETNS */
/* #undef HAVE_KQUEUE_TIMERS */
/* #undef HAVE_POSIX_TIMERS */

/* Endianess */
/* #undef WORDS_BIGENDIAN */
/* #undef HAVE_ENDIAN_CONVERSION_MACROS */

/* Type sizes */
/* #undef SIZEOF_VOIDP */
/* #undef SIZEOF_CHARP */
/* #undef SIZEOF_LONG */
/* #undef SIZEOF_SHORT */
/* #undef SIZEOF_INT */
/* #undef SIZEOF_LONG_LONG */
/* #undef SIZEOF_TIME_T */
/* #undef HAVE_ULONG */
/* #undef HAVE_U_INT32_T */
/* #undef HAVE_TM_GMTOFF */

/* Support for tagging symbols with __attribute__((visibility("hidden"))) */
/* #undef HAVE_VISIBILITY_HIDDEN */

/* Code tests*/
/* #undef HAVE_CLOCK_GETTIME */
/* #undef HAVE_CLOCK_REALTIME */
/* #undef STACK_DIRECTION */
/* #undef TIME_WITH_SYS_TIME */
/* #undef NO_FCNTL_NONBLOCK */
/* #undef HAVE_PAUSE_INSTRUCTION */
/* #undef HAVE_FAKE_PAUSE_INSTRUCTION */
/* #undef HAVE_HMT_PRIORITY_INSTRUCTION */
/* #undef HAVE_ABI_CXA_DEMANGLE */
/* #undef HAVE_BUILTIN_UNREACHABLE */
/* #undef HAVE_BUILTIN_EXPECT */
/* #undef HAVE_BUILTIN_STPCPY */
/* #undef HAVE_GCC_SYNC_BUILTINS */
/* #undef HAVE_VALGRIND */
/* #undef HAVE_SYS_GETTID */
/* #undef HAVE_PTHREAD_GETTHREADID_NP */
/* #undef HAVE_PTHREAD_THREADID_NP */
/* #undef HAVE_INTEGER_PTHREAD_SELF */
/* #undef HAVE_PTHREAD_SETNAME_NP */

/* IPV6 */
/* #undef HAVE_NETINET_IN6_H */
/* #undef HAVE_STRUCT_IN6_ADDR */

/*
 * Platform specific CMake files
 */
#define MACHINE_TYPE ""
/* #undef LINUX_ALPINE */
/* #undef LINUX_SUSE */
/* #undef LINUX_RHEL6 */
/* #undef LINUX_RHEL7 */
/* #undef LINUX_RHEL8 */
/* #undef HAVE_LINUX_LARGE_PAGES */
/* #undef HAVE_SOLARIS_LARGE_PAGES */
/* #undef HAVE_SOLARIS_ATOMIC */
/* #undef WITH_SYSTEMD_DEBUG */
#define SYSTEM_TYPE "linux"
/* This should mean case insensitive file system */
/* #undef FN_NO_CASE_SENSE */
/* #undef APPLE_ARM */
/* #undef HAVE_BUILD_ID_SUPPORT */

/*
 * From main CMakeLists.txt
 */
/* #undef CHECK_ERRMSG_FORMAT */
/* #undef MAX_INDEXES */
#define MAX_INDEXES 1024
/* #undef WITH_INNODB_MEMCACHED */
/* #undef ENABLE_MEMCACHED_SASL */
/* #undef ENABLE_MEMCACHED_SASL_PWDB */
/* #undef ENABLED_PROFILING */
/* #undef HAVE_ASAN */
/* #undef HAVE_LSAN */
/* #undef HAVE_UBSAN */
/* #undef HAVE_TSAN */
/* #undef ENABLED_LOCAL_INFILE */
/* #undef KERBEROS_LIB_CONFIGURED */
/* #undef SCRAM_LIB_CONFIGURED */
/* #undef WITH_HYPERGRAPH_OPTIMIZER */
/* #undef KERBEROS_LIB_SSPI */

/* Lock Order */
/* #undef WITH_LOCK_ORDER */

/* Character sets and collations */
/* #undef DEFAULT_MYSQL_HOME */
/* #undef SHAREDIR */
/* #undef DEFAULT_BASEDIR */
/* #undef MYSQL_DATADIR */
/* #undef MYSQL_KEYRINGDIR */
/* #undef DEFAULT_CHARSET_HOME */
/* #undef PLUGINDIR */
/* #undef DEFAULT_SYSCONFDIR */
/* #undef DEFAULT_TMPDIR */
/* #undef MYSQL_ICU_DATADIR */
/* #undef ICUDT_DIR */
/*
 * Readline
 */
/* #undef HAVE_MBSTATE_T */
/* #undef HAVE_LANGINFO_CODESET */
/* #undef HAVE_WCSDUP */
/* #undef HAVE_WCHAR_T */
/* #undef HAVE_WINT_T */
/* #undef HAVE_CURSES_H */
/* #undef HAVE_NCURSES_H */
/* #undef USE_LIBEDIT_INTERFACE */
/* #undef HAVE_HIST_ENTRY */
/* #undef USE_NEW_EDITLINE_INTERFACE */
/* #undef EDITLINE_HAVE_COMPLETION_CHAR */
/* #undef EDITLINE_HAVE_COMPLETION_INT */


/*
 * Libedit
 */
/* #undef HAVE_GETLINE */
/* #undef HAVE___SECURE_GETENV */
/* #undef HAVE_SECURE_GETENV */
/* #undef HAVE_VIS */
/* #undef HAVE_UNVIS */
/* #undef HAVE_GETPW_R_DRAFT */
/* #undef HAVE_GETPW_R_POSIX */

/*
 * Character sets
 */
/* #undef MYSQL_DEFAULT_CHARSET_NAME */
/* #undef MYSQL_DEFAULT_COLLATION_NAME */

/*
 * Performance schema
 */
/* #undef WITH_PERFSCHEMA_STORAGE_ENGINE */
/* #undef DISABLE_PSI_THREAD */
/* #undef DISABLE_PSI_MUTEX */
/* #undef DISABLE_PSI_RWLOCK */
/* #undef DISABLE_PSI_COND */
/* #undef DISABLE_PSI_FILE */
/* #undef DISABLE_PSI_TABLE */
/* #undef DISABLE_PSI_SOCKET */
/* #undef DISABLE_PSI_STAGE */
/* #undef DISABLE_PSI_STATEMENT */
/* #undef DISABLE_PSI_SP */
/* #undef DISABLE_PSI_PS */
/* #undef DISABLE_PSI_IDLE */
/* #undef DISABLE_PSI_ERROR */
/* #undef DISABLE_PSI_STATEMENT_DIGEST */
/* #undef DISABLE_PSI_METADATA */
/* #undef DISABLE_PSI_MEMORY */
/* #undef DISABLE_PSI_TRANSACTION */

/*
 * MySQL version
 */
#define MYSQL_VERSION_MAJOR 8
#define MYSQL_VERSION_MINOR 0
#define MYSQL_VERSION_PATCH 0
#define MYSQL_VERSION_EXTRA "y"
#define PACKAGE "mysql"
#define PACKAGE_VERSION "8.0.0-y"
#define VERSION "8.0.0-y"

/*
 * CPU info
 */
/* #undef CPU_LEVEL1_DCACHE_LINESIZE */
/* #undef CPU_PAGE_SIZE */

/*
 * NDB
 */
/* #undef HAVE_GETRLIMIT */
/* #undef WITH_NDBCLUSTER_STORAGE_ENGINE */
/* #undef HAVE_PTHREAD_SETSCHEDPARAM */

/*
 * Other
 */
/* #undef EXTRA_DEBUG */
/* #undef HANDLE_FATAL_SIGNALS */

/*
 * Hardcoded values needed by libevent/NDB/memcached
 */
#define HAVE_FCNTL_H 1
#define HAVE_GETADDRINFO 1
#define HAVE_INTTYPES_H 1
#define HAVE_SIGNAL_H 1
#define HAVE_STDARG_H 1
#define HAVE_STDINT_H 1
#define HAVE_STDLIB_H 1
#define HAVE_STRTOK_R 1
#define HAVE_STRTOLL 1
#define HAVE_SYS_STAT_H 1
#define HAVE_SYS_TYPES_H 1
#define SIZEOF_CHAR 1

/* For --secure-file-priv */
/* #undef DEFAULT_SECURE_FILE_PRIV_DIR */
/* #undef HAVE_LIBNUMA */

/* For default value of --early_plugin_load */
/* #undef DEFAULT_EARLY_PLUGIN_LOAD */

/* For default value of --partial_revokes */
#define DEFAULT_PARTIAL_REVOKES

#define SO_EXT ".so"


/* From libmysql/CMakeLists.txt */
/* #undef HAVE_UNIX_DNS_SRV */
/* #undef HAVE_WIN32_DNS_SRV */

/* ARM crc32 support */
/* #undef HAVE_ARMV8_CRC32_INTRINSIC */

#endif
