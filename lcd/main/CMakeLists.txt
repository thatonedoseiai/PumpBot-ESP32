set(srcs
    "board_config.c"
    "ILIDriver.c"
    "oam.c"
    "pwm_fade.c"
    "menus/bootloader.c"
    "lua_defs.c"
    "pb_main.c"
    "rotenc.c"
    "utf8.c"
    "button.c"
    "settings.c"
    "http.c"
    )

idf_component_register(SRCS ${srcs}
                    INCLUDE_DIRS 
                        "./includes"
                        "./freetype2"
)

# littlefs_create_partition_image(filesystem mainfs)

# add_library(freetype STATIC IMPORTED GLOBAL)
# set_property(TARGET freetype PROPERTY IMPORTED_LOCATION "${COMPONENT_DIR}/libfreetype.a")
# target_link_libraries(${COMPONENT_TARGET} freetype)
add_prebuilt_library(freetype "${COMPONENT_DIR}/libfreetype.a" PRIV_REQUIRES esp_rom)
idf_build_set_property(LINK_OPTIONS "-Wl,--defsym=_setjmp=setjmp" APPEND)
target_link_libraries(${COMPONENT_TARGET} INTERFACE freetype)
