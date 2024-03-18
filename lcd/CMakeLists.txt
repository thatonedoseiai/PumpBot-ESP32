# The following lines of boilerplate have to be in your project's CMakeLists
# in this exact order for cmake to work correctly
cmake_minimum_required(VERSION 3.16)

# set(EXTRA_COMPONENT_DIRS $ENV{HOME}/../Workspace/esp32/PumpBot-ESP32/lcd/components/esp-idf-lua)
set(EXTRA_COMPONENT_DIRS ./components/esp-idf-lua)
include($ENV{IDF_PATH}/tools/cmake/project.cmake)
project(pumpbot)
littlefs_create_partition_image(filesystem ./main/mainfs FLASH_IN_PROJECT)
