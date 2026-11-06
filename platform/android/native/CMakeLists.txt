cmake_minimum_required(VERSION 3.22.1)
project("audiobridge_native")

add_library(audiobridge_native SHARED
    OboeAudioSink.cpp
)

target_include_directories(audiobridge_native PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}
)

find_library(log-lib log)
find_library(aaudio-lib aaudio)

target_link_libraries(audiobridge_native
    ${log-lib}
    ${aaudio-lib}
)
