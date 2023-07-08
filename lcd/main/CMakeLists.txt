set(srcs "pretty_effect.c"
    "spi_master_example_main.c"
    "ILIDriver.c"
    "decode_image.c"
    )

idf_component_register(SRCS ${srcs}
                    INCLUDE_DIRS "."
                    EMBED_FILES image.jpg)

add_library(freetype STATIC IMPORTED GLOBAL)
set_property(TARGET freetype PROPERTY IMPORTED_LOCATION "${COMPONENT_DIR}/precompiled_test.o")
target_link_libraries(${COMPONENT_TARGET} freetype)
