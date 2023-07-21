#include <stdio.h>
#include <rom/ets_sys.h>

init_sprite(sprite24b sprite, uint24_RGB *bitmap, uint16_t posX, uint16_t posY, uint16_t sizeX, uint16_t sizeY, bool flipX, bool flipY) {
    sprite = malloc(sizeof(sprite24b));
    sprite.bitmap = malloc(posX * posY * sizeof(uint24_RGB));
    sprite.posX = posX;
    sprite.posY = posY;
    sprite.sizeX = sizeX;
    sprite.sizeY = sizeY;
    sprite.flipX = flipX;
    sprite.flipY = flipY;
}