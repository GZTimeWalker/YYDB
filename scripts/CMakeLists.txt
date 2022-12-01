# Copyright (c) 2006, 2022, Oracle and/or its affiliates.
#
# This program is free software; you can redistribute it and/or modify
# it under the terms of the GNU General Public License, version 2.0,
# as published by the Free Software Foundation.
#
# This program is also distributed with certain software (including
# but not limited to OpenSSL) that is licensed under separate terms,
# as designated in a particular file or component or in included license
# documentation.  The authors of MySQL hereby grant you an additional
# permission to link the program and your derivative works with the
# separately licensed software that they have included with MySQL.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License, version 2.0, for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program; if not, write to the Free Software
# Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301  USA

SET(Y_ENGINE_PLUGIN_DYNAMIC "ha_yengine")
SET(Y_ENGINE_SOURCES
  ha_wapper.cc
  yengine.cc
  )

FIND_LIBRARY(YENGINE_LIB yengine .)
ADD_DEFINITIONS(-DMYSQL_DYNAMIC_PLUGIN)
ADD_DEFINITIONS(-DLOG_SUBSYSTEM_TAG="Y-Engine")

IF(WITH_Y_ENGINE_STORAGE_ENGINE AND NOT WITHOUT_Y_ENGINE_STORAGE_ENGINE)
  MYSQL_ADD_PLUGIN(yengine ${Y_ENGINE_SOURCES} STORAGE_ENGINE DEFAULT     LINK_LIBRARIES ${YENGINE_LIB})
ELSEIF(NOT WITHOUT_Y_ENGINE_STORAGE_ENGINE)
  MYSQL_ADD_PLUGIN(yengine ${Y_ENGINE_SOURCES} STORAGE_ENGINE MODULE_ONLY LINK_LIBRARIES ${YENGINE_LIB})
ENDIF()
