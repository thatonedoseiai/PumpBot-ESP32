#pragma once
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "esp_system.h"
#include "driver/spi_master.h"
#include "driver/gpio.h"
#include <stdint.h>
#include <stdbool.h>
#include "esp_err.h"

#ifdef __cplusplus
extern "C" {
#endif

/*
 This code displays some fancy graphics on the 320x240 LCD on an ESP-WROVER_KIT board.
 This example demonstrates the use of both spi_device_transmit as well as
 spi_device_queue_trans/spi_device_get_trans_result and pre-transmit callbacks.

 Some info about the ILI9341/ST7789V: It has an C/D line, which is connected to a GPIO here. It expects this
 line to be low for a command and high for data. We use a pre-transmit callback here to control that
 line: every transaction has as the user-definable argument the needed state of the D/C line and just
 before the transaction is sent, the callback will set this line to the correct state.
*/

#define LCD_HOST	HSPI_HOST

#define PIN_NUM_MISO 12
#define PIN_NUM_MOSI 13
#define PIN_NUM_CLK  14
#define PIN_NUM_CS   15

#define PIN_NUM_DC   27
#define PIN_NUM_RST  2
#define PIN_NUM_BCKL 5

//To speed up transfers, every SPI transfer sends a bunch of lines. This define specifies how many. More means more memory use,
//but less overhead for setting up / finishing transfers. Make sure 240 is dividable by this.
#define PARALLEL_LINES 16

typedef struct uint24_RGB_ {
	uint8_t pixelR;
	uint8_t pixelG;
	uint8_t pixelB;
} uint24_RGB;

/* The LCD needs a bunch of command/argument values to be initialized. They are stored in this struct. */
typedef struct {
	uint8_t cmd;
	uint8_t data[16];
	uint8_t databytes; //No of data in data; bit 7 = delay after set; 0xFF = end of cmds.
} lcd_init_cmd_t;

//Place data into DRAM. Constant data gets placed into DROM by default, which is not accessible by DMA.
// {{{
DRAM_ATTR static const lcd_init_cmd_t ili_init_cmds[]={
	/* Power contorl B, power control = 0, DC_ENA = 1 */
	{0xCF, {0x00, 0x83, 0X30}, 3},
	/* Power on sequence control,
	 * cp1 keeps 1 frame, 1st frame enable
	 * vcl = 0, ddvdh=3, vgh=1, vgl=2
	 * DDVDH_ENH=1
	 */
	{0xED, {0x64, 0x03, 0X12, 0X81}, 4},
	/* Driver timing control A,
	 * non-overlap=default +1
	 * EQ=default - 1, CR=default
	 * pre-charge=default - 1
	 */
	{0xE8, {0x85, 0x01, 0x79}, 3},
	/* Power control A, Vcore=1.6V, DDVDH=5.6V */
	{0xCB, {0x39, 0x2C, 0x00, 0x34, 0x02}, 5},
	/* Pump ratio control, DDVDH=2xVCl */
	{0xF7, {0x20}, 1},
	/* Driver timing control, all=0 unit */
	{0xEA, {0x00, 0x00}, 2},
	/* Power control 1, GVDD=4.75V */
	{0xC0, {0x26}, 1},
	/* Power control 2, DDVDH=VCl*2, VGH=VCl*7, VGL=-VCl*3 */
	{0xC1, {0x11}, 1},
	/* VCOM control 1, VCOMH=4.025V, VCOML=-0.950V */
	{0xC5, {0x35, 0x3E}, 2},
	/* VCOM control 2, VCOMH=VMH-2, VCOML=VML-2 */
	{0xC7, {0xBE}, 1},
	/* Memory access contorl, MX=MY=0, MV=1, ML=0, BGR=1, MH=0 */
	{0x36, {0x28}, 1},
	/* Pixel format, 18bits/pixel for RGB/MCU interface */
	{0x3A, {0x66}, 1},
	/* Frame rate control, f=fosc, 70Hz fps */
	{0xB1, {0x00, 0x10}, 2}, //currently 0x10 - 119Hz
	/* Enable 3G, disabled */
	{0xF2, {0x08}, 1},
	/* Gamma set, curve 1 */
	{0x26, {0x01}, 1},
	/* Positive gamma correction */
	{0xE0, {0x1F, 0x1A, 0x18, 0x0A, 0x0F, 0x06, 0x45, 0X87, 0x32, 0x0A, 0x07, 0x02, 0x07, 0x05, 0x00}, 15},
	/* Negative gamma correction */
	{0XE1, {0x00, 0x25, 0x27, 0x05, 0x10, 0x09, 0x3A, 0x78, 0x4D, 0x05, 0x18, 0x0D, 0x38, 0x3A, 0x1F}, 15},
	/* Column address set, SC=0, EC=0xEF */
	{0x2A, {0x00, 0x00, 0x00, 0xEF}, 4},
	/* Page address set, SP=0, EP=0x013F */
	{0x2B, {0x00, 0x00, 0x01, 0x3f}, 4},
	/* Memory write */
	{0x2C, {0}, 0},
	/* Entry mode set, Low vol detect disabled, normal display */
	{0xB7, {0x07}, 1},
	/* Display function control */
	{0xB6, {0x0A, 0x82, 0x27, 0x00}, 4},
	/* Sleep out */
	{0x11, {0}, 0x80},
	/* Display on */
	{0x29, {0}, 0x80},
	{0, {0}, 0xff},
};
// }}}

/* Send a command to the LCD. Uses spi_device_polling_transmit, which waits
 * until the transfer is complete.
 *
 * Since command transactions are usually small, they are handled in polling
 * mode for higher speed. The overhead of interrupt transactions is more than
 * just waiting for the transaction to complete.
 */
void lcd_cmd(spi_device_handle_t spi, const uint8_t cmd, bool keep_cs_active);

/* Send data to the LCD. Uses spi_device_polling_transmit, which waits until the
 * transfer is complete.
 *
 * Since data transactions are usually small, they are handled in polling
 * mode for higher speed. The overhead of interrupt transactions is more than
 * just waiting for the transaction to complete.
 */
void lcd_data(spi_device_handle_t spi, const uint8_t *data, int len);

//This function is called (in irq context!) just before a transmission starts. It will
//set the D/C line to the value indicated in the user field.
void lcd_spi_pre_transfer_callback(spi_transaction_t *t);

uint32_t lcd_get_id(spi_device_handle_t spi);

//Initialize the display
void lcd_init(spi_device_handle_t spi);


/* To send a set of lines we have to send a command, 2 data bytes, another command, 2 more data bytes and another command
 * before sending the line data itself; a total of 6 transactions. (We can't put all of this in just one transaction
 * because the D/C line needs to be toggled in the middle.)
 * This routine queues these commands up as interrupt transactions so they get
 * sent faster (compared to calling spi_device_transmit several times), and at
 * the mean while the lines for next transactions can get calculated.
 */
void send_lines(spi_device_handle_t spi, int ypos, uint24_RGB *linedata);

/*
 * fills the whole screen with one color.
 */
void send_color(spi_device_handle_t spi, uint24_RGB color);

/*
 * draws a sprite at an x and y coordinate.
 */
void draw_sprite(spi_device_handle_t spi, uint16_t x, uint16_t y, uint16_t width, uint16_t height, uint24_RGB* bitmap);

/* wait for line transfers to be done and check their status.
 *
 * Call this at the end, after the next frame is finished calculating.
 * See the comments at the end of ILIDriver.c->send_lines
 */
void send_line_finish(spi_device_handle_t spi);

#ifdef __cplusplus
}
#endif

