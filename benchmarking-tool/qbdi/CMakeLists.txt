cmake_minimum_required (VERSION 3.2)
project(QBDITemplate)

find_package(QBDIAARCH64 REQUIRED)
find_package(QBDIPreloadAARCH64 REQUIRED)

add_library(qbdi_tracer SHARED icount.c)
target_link_libraries(qbdi_tracer QBDIPreload::AARCH64::QBDIPreload QBDI::AARCH64::QBDI_static)
