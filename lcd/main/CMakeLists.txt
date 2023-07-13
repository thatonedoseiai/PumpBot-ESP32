set(srcs "pretty_effect.c"
    "pb_main.c"
    "ILIDriver.c"
    "decode_image.c"
    )

idf_component_register(SRCS ${srcs}
                    INCLUDE_DIRS 
                        "."
                        # "./freetype"
                        "./freetype2"
                    EMBED_FILES
                        image.jpg
                        MeiryoUI.ttf
)

# add_library(freetype STATIC IMPORTED GLOBAL)
# set_property(TARGET freetype PROPERTY IMPORTED_LOCATION "${COMPONENT_DIR}/freetype/esp_freetype.a")
# set_property(TARGET freetype PROPERTY IMPORTED_LOCATION "${COMPONENT_DIR}/libfreetype.a")
# target_link_libraries(${COMPONENT_TARGET} freetype)
add_prebuilt_library(freetype "${COMPONENT_DIR}/libfreetype.a" PRIV_REQUIRES esp_rom)
idf_build_set_property(LINK_OPTIONS "-Wl,--defsym=_setjmp=setjmp" APPEND)
target_link_libraries(${COMPONENT_TARGET} INTERFACE freetype)
