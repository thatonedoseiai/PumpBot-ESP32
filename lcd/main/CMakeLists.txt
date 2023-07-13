set(srcs "pretty_effect.c"
    "pb_main.c"
    "ILIDriver.c"
    "decode_image.c"
    )

idf_component_register(SRCS ${srcs}
                    INCLUDE_DIRS 
                        "."
                        "./freetype"
                    EMBED_FILES
                        image.jpg
                        MeiryoUI.ttf)

add_library(freetype STATIC IMPORTED GLOBAL)
set_property(TARGET freetype PROPERTY IMPORTED_LOCATION "${COMPONENT_DIR}/freetype/esp_freetype.a")
target_link_libraries(${COMPONENT_TARGET} freetype)
