set(srcs
    "ILIDriver.c"
    "oam.c"
    "pb_main.c"
    )

idf_component_register(SRCS ${srcs}
                    INCLUDE_DIRS 
                        "."
                        "./freetype2"
                    EMBED_FILES
                        MeiryoUImin.ttf
)

# add_library(freetype STATIC IMPORTED GLOBAL)
# set_property(TARGET freetype PROPERTY IMPORTED_LOCATION "${COMPONENT_DIR}/libfreetype.a")
# target_link_libraries(${COMPONENT_TARGET} freetype)
add_prebuilt_library(freetype "${COMPONENT_DIR}/libfreetype.a" PRIV_REQUIRES esp_rom)
idf_build_set_property(LINK_OPTIONS "-Wl,--defsym=_setjmp=setjmp" APPEND)
target_link_libraries(${COMPONENT_TARGET} INTERFACE freetype)
