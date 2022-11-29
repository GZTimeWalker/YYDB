/* Copyright Abandoned 1996,1999 TCX DataKonsult AB & Monty Program KB
   & Detron HB, 1996, 1999-2004, 2007 MySQL AB.
   This file is public domain and comes with NO WARRANTY of any kind
*/

/* Version numbers for protocol & mysqld */

#ifndef _mysql_version_h
#define _mysql_version_h

#define PROTOCOL_VERSION            10
#define MYSQL_SERVER_VERSION       "8.0.0"
#define MYSQL_BASE_VERSION         "mysqld-8.0"
#define MYSQL_SERVER_SUFFIX_DEF    ""
#define MYSQL_VERSION_ID            8
#define MYSQL_PORT                  3306
#define MYSQL_ADMIN_PORT
#define MYSQL_PORT_DEFAULT
#define MYSQL_UNIX_ADDR            ""
#define MYSQL_CONFIG_NAME          "my"
#define MYSQL_PERSIST_CONFIG_NAME  "mysqld-auto"
#define MYSQL_COMPILATION_COMMENT  ""
#define MYSQL_COMPILATION_COMMENT_SERVER  ""
#define LIBMYSQL_VERSION           ""
#define LIBMYSQL_VERSION_ID

#ifndef LICENSE
#define LICENSE                     GPL
#endif /* LICENSE */

#endif /* _mysql_version_h */
