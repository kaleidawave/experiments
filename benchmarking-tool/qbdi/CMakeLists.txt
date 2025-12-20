cmake_minimum_required (VERSION 3.2)
project(QBDITemplate)

if(WIN32)
    # ${QBDI_ARCH}
    find_package(QBDI REQUIRED)

    add_library(qbdi_tracer SHARED icount.c)
    # ::${QBDI_ARCH}
    target_link_libraries(qbdi_tracer
        QBDI::QBDI_static
    )
else()
    if(CMAKE_SYSTEM_PROCESSOR MATCHES "^(x86_64|AMD64)$")
        set(QBDI_ARCH X86_64)
    elseif(CMAKE_SYSTEM_PROCESSOR MATCHES "^(aarch64|ARM64|arm64)$")
        set(QBDI_ARCH AARCH64)
    else()
        message(FATAL_ERROR
            "Unsupported architecture: ${CMAKE_SYSTEM_PROCESSOR}"
        )
    endif()

    find_package(QBDI${QBDI_ARCH} REQUIRED)
    find_package(QBDIPreload${QBDI_ARCH} REQUIRED)

    add_library(qbdi_tracer SHARED icount.c)
    target_link_libraries(qbdi_tracer
        QBDIPreload::${QBDI_ARCH}::QBDIPreload
        QBDI::${QBDI_ARCH}::QBDI_static
    )
endif()
