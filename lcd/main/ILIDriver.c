#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <inttypes.h>
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "esp_system.h"
#include "driver/spi_master.h"
#include "driver/gpio.h"
#include <rom/ets_sys.h>

#include "ILIDriver.h"
// #include "pretty_effect.h"

uint24_RGB* framebuf = NULL;

void lcd_cmd(spi_device_handle_t spi, const uint8_t cmd, bool keep_cs_active) {
	esp_err_t ret;
	spi_transaction_t t;
	memset(&t, 0, sizeof(t));		//Zero out the transaction
	t.length=8;						//Command is 8 bits
	t.tx_buffer=&cmd;				//The data is the cmd itself
	t.user=(void*)0;				//D/C needs to be set to 0
	if (keep_cs_active) {
	  t.flags = SPI_TRANS_CS_KEEP_ACTIVE;		//Keep CS active after data transfer
	}
	ret=spi_device_polling_transmit(spi, &t);	//Transmit!
	assert(ret==ESP_OK);						//Should have had no issues.
}

void lcd_data(spi_device_handle_t spi, const uint8_t *data, int len) {
	esp_err_t ret;
	spi_transaction_t t;
	if (len==0) return;					//no need to send anything
	memset(&t, 0, sizeof(t));			//Zero out the transaction
	t.length=len*8;						//Len is in bytes, transaction length is in bits.
	t.tx_buffer=data;					//Data
	t.user=(void*)1;					//D/C needs to be set to 1
	ret=spi_device_polling_transmit(spi, &t);  //Transmit!
	assert(ret==ESP_OK);				//Should have had no issues.
}

void lcd_spi_pre_transfer_callback(spi_transaction_t *t) {
	int dc=(int)t->user;
	gpio_set_level(PIN_NUM_DC, dc);
}

uint32_t lcd_get_id(spi_device_handle_t spi) {
	// When using SPI_TRANS_CS_KEEP_ACTIVE, bus must be locked/acquired
	spi_device_acquire_bus(spi, portMAX_DELAY);

	//get_id cmd
	lcd_cmd(spi, 0x04, true);

	spi_transaction_t t;
	memset(&t, 0, sizeof(t));
	t.length=8*3;
	t.flags = SPI_TRANS_USE_RXDATA;
	t.user = (void*)1;

	esp_err_t ret = spi_device_polling_transmit(spi, &t);
	assert( ret == ESP_OK );

	// Release bus
	spi_device_release_bus(spi);

	return *(uint32_t*)t.rx_data;
}

void lcd_init(spi_device_handle_t spi) {
	// gpio_set_level(PIN_NUM_BCKL, 1); //Enable backlight
	int cmd=0;
	const lcd_init_cmd_t* lcd_init_cmds;
    // framebuf = malloc(320*240*sizeof(uint24_RGB));
    // memset(framebuf, 0, 320*240*sizeof(uint24_RGB));

	//Initialize non-SPI GPIOs
	gpio_config_t io_conf = {};
	io_conf.pin_bit_mask = ((1ULL<<PIN_NUM_DC) | (1ULL<<PIN_NUM_RST));
	// | (1ULL<<PIN_NUM_BCKL));
	io_conf.mode = GPIO_MODE_OUTPUT;
	io_conf.pull_up_en = true;
	gpio_config(&io_conf);

	//Reset the display
	gpio_set_level(PIN_NUM_RST, 0);
	vTaskDelay(100 / portTICK_PERIOD_MS);
	gpio_set_level(PIN_NUM_RST, 1);
	vTaskDelay(100 / portTICK_PERIOD_MS);

	printf("LCD ILI9341 initialized!\n");
	lcd_init_cmds = ili_init_cmds;


	//Send all the commands
	while (lcd_init_cmds[cmd].databytes!=0xff) {
		lcd_cmd(spi, lcd_init_cmds[cmd].cmd, false);
		lcd_data(spi, lcd_init_cmds[cmd].data, lcd_init_cmds[cmd].databytes&0x1F);
		if (lcd_init_cmds[cmd].databytes&0x80) {
			vTaskDelay(100 / portTICK_PERIOD_MS);
		}
		cmd++;
	}
}

void send_lines(spi_device_handle_t spi, int ypos, uint24_RGB *linedata, int num_cols) {
	esp_err_t ret;
	int x;
	//Transaction descriptors. Declared static so they're not allocated on the stack; we need this memory even when this
	//function is finished because the SPI driver needs access to it even while we're already calculating the next line.
	static spi_transaction_t trans[6];
    if(num_cols > PARALLEL_LINES) {
        num_cols = PARALLEL_LINES;
    }

	//In theory, it's better to initialize trans and data only once and hang on to the initialized
	//variables. We allocate them on the stack, so we need to re-init them each call.
	for (x=0; x<6; x++) {
		memset(&trans[x], 0, sizeof(spi_transaction_t));
		if ((x&1)==0) {
			//Even transfers are commands
			trans[x].length=8;
			trans[x].user=(void*)0;
		} else {
			//Odd transfers are data
			trans[x].length=8*4;
			trans[x].user=(void*)1;
		}
		trans[x].flags=SPI_TRANS_USE_TXDATA;
	}
	trans[0].tx_data[0]=0x2A;						//Column Address Set
	trans[1].tx_data[0]=0;							//Start Col High
	trans[1].tx_data[1]=0;							//Start Col Low
	trans[1].tx_data[2]=(240)>>8;					//End Col High
	trans[1].tx_data[3]=(240)&0xff;					//End Col Low
	trans[2].tx_data[0]=0x2B;						//Page address set
	trans[3].tx_data[0]=ypos>>8;					//Start page high
	trans[3].tx_data[1]=ypos&0xff;					//start page low
	trans[3].tx_data[2]=(ypos+num_cols)>>8;	//end page high
	trans[3].tx_data[3]=(ypos+num_cols)&0xff;	//end page low
	trans[4].tx_data[0]=0x2C;						//memory write
	trans[5].tx_buffer=linedata;					//finally send the line data
	trans[5].length=240*3*8*num_cols;			//Data length, in bits
	trans[5].flags=0;								//undo SPI_TRANS_USE_TXDATA flag

	//Queue all transactions.
	for (x=0; x<6; x++) {
		ret=spi_device_queue_trans(spi, &trans[x], portMAX_DELAY);
		if(ret!=ESP_OK) {
            ets_printf("%d %d %d\n", ret, ESP_OK, ret==ESP_OK);
            assert(false);
        }
	}

	//When we are here, the SPI driver is busy (in the background) getting the transactions sent. That happens
	//mostly using DMA, so the CPU doesn't have much to do here. We're not going to wait for the transaction to
	//finish because we may as well spend the time calculating the next line. When that is done, we can call
	//send_line_finish, which will wait for the transfers to be done and check their status.
}

void buffer_fillcolor(uint24_RGB* col) {
    if(framebuf == NULL) {
        framebuf = malloc(320*240*sizeof(uint24_RGB));
        memset(framebuf, 0, 320*240*sizeof(uint24_RGB));
    }
    for(int i=0;i<320*240;i++) {
        framebuf[i] = *col;
    }
}

void buffer_sprite(uint16_t x, uint16_t y, uint16_t width, uint16_t height, uint24_RGB* bitmap) {
    if(framebuf == NULL) {
        framebuf = malloc(320*240*sizeof(uint24_RGB));
        memset(framebuf, 0, 320*240*sizeof(uint24_RGB));
    }

    for(int i=0;i<width;i++) {
        for(int j=0;j<height;j++) {
            framebuf[(i+x)*240+(j+y)] = bitmap[i*height+j];
        }
    }
}

void draw_sprite(spi_device_handle_t spi, uint16_t sx, uint16_t y, uint16_t width, uint16_t height, uint24_RGB* bitmap) {
	esp_err_t ret;
	int x;
	static spi_transaction_t trans[6];
	
	if(sx > 320)
		return;

	for (x=0; x<6; x++) {
		memset(&trans[x], 0, sizeof(spi_transaction_t));
		if ((x&1)==0) {
			//Even transfers are commands
			trans[x].length=8;
			trans[x].user=(void*)0;
		} else {
			//Odd transfers are data
			trans[x].length=8*4;
			trans[x].user=(void*)1;
		}
		trans[x].flags=SPI_TRANS_USE_TXDATA;
	}

	trans[0].tx_data[0]=0x2A;								//Column Address Set
	trans[1].tx_data[0]=y>>8;								//Start Col High
	trans[1].tx_data[1]=y&0xff;							//Start Col Low
	trans[1].tx_data[2]=(y+height-1)>>8;					//End Col High
	trans[1].tx_data[3]=(y+height-1)&0xff;					//End Col Low
	trans[2].tx_data[0]=0x2B;								//Page address set
	trans[3].tx_data[0]=sx>>8;								//Start page high
	trans[3].tx_data[1]=sx&0xff;								//start page low
	trans[3].tx_data[2]=(sx+width-1)>>8;					//end page high
	trans[3].tx_data[3]=(sx+width-1)&0xff;					//end page low
	trans[4].tx_data[0]=0x2C;								//memory write
	trans[5].tx_buffer=bitmap;								//finally send the line data
	trans[5].length=width*height*sizeof(uint24_RGB) << 3;	//Data length, in bits
	trans[5].flags=0;										//undo SPI_TRANS_USE_TXDATA flag

	//Queue all transactions.
	for (x=0; x<6; x++) {
		ret=spi_device_queue_trans(spi, &trans[x], portMAX_DELAY);
		assert(ret==ESP_OK);
	}

}

void scroll_screen(spi_device_handle_t spi, uint16_t value) {
    const static uint8_t data_33[6] = {0,0,1,0x40,0,0};
    static uint8_t data_37[2];
    data_37[0]=value>>8;
    data_37[1]=value&0xff;

    lcd_cmd(spi, 0x33, false);
    lcd_data(spi, data_33, 6);
    lcd_cmd(spi, 0x37, false);
    lcd_data(spi, data_37, 2);
}

void send_color(spi_device_handle_t spi, uint24_RGB* color) {
	int pixelCount = 400*PARALLEL_LINES;
	int bytelength = 3*pixelCount;
	uint24_RGB* colorbuf = malloc(sizeof(uint24_RGB) * bytelength);
	for (int i = 0; i < pixelCount; ++i)
		colorbuf[i] = *color; 
	for(int ypos=0;ypos<240;ypos+=PARALLEL_LINES) {
		esp_err_t ret;
		int x;
		static spi_transaction_t trans[6];
        // ets_printf("ypos: %d\n", ypos);

		for (x=0; x<6; x++) {
			memset(&trans[x], 0, sizeof(spi_transaction_t));
			if ((x&1)==0) {
				//Even transfers are commands
				trans[x].length=8;
				trans[x].user=(void*)0;
			} else {
				//Odd transfers are data
				trans[x].length=8*4;
				trans[x].user=(void*)1;
			}
			trans[x].flags=SPI_TRANS_USE_TXDATA;
		}
		trans[0].tx_data[0]=0x2A;		   //Column Address Set
		trans[1].tx_data[0]=ypos>>8;			  //Start Col High
		trans[1].tx_data[1]=ypos&0xff;			  //Start Col Low
		trans[1].tx_data[2]=(ypos+PARALLEL_LINES)>>8;	   //End Col High
		trans[1].tx_data[3]=(ypos+PARALLEL_LINES)&0xff;	 //End Col Low
		trans[2].tx_data[0]=0x2B;		   //Page address set
		trans[3].tx_data[0]=0;		//Start page high
		trans[3].tx_data[1]=0;	  //start page low
		trans[3].tx_data[2]=(320)>>8;	//end page high
		trans[3].tx_data[3]=(320)&0xff;  //end page low
		trans[4].tx_data[0]=0x2C;		   //memory write
		trans[5].tx_buffer=colorbuf;		//finally send the color data
		trans[5].length=bytelength << 3;		  //Data length, in bits
		trans[5].flags=0; //undo SPI_TRANS_USE_TXDATA flag
		//Queue all transactions.
		for (x=0; x<6; x++) {
			ret=spi_device_queue_trans(spi, &trans[x], portMAX_DELAY);
			assert(ret==ESP_OK);
		}
		send_line_finish(spi);
	}
	free(colorbuf);
}

void scroll_buffer(spi_device_handle_t spi, int screenoffset, bool resetScroll) {
	// esp_err_t ret;
	// int x;
	// static spi_transaction_t trans[10];
    static int i = 0;
    // int num_cols;
    // const static uint8_t data_33[6] = {0,0,1,0x40,0,0};
    if(resetScroll) {
        i = 0;
        scroll_screen(spi, 0);
    }

    while(i<screenoffset) {
        send_lines(spi, i, framebuf+(i*240), screenoffset-i < PARALLEL_LINES ? screenoffset-i : PARALLEL_LINES);
        scroll_screen(spi, 320-i);
        send_line_finish(spi);
        // send_scroll_finish(spi);
        i+=screenoffset-i < PARALLEL_LINES ? screenoffset-i : PARALLEL_LINES;
    }
}

void send_scroll_finish(spi_device_handle_t spi) {
	spi_transaction_t *rtrans;
	esp_err_t ret;
	//Wait for all 10 transactions to be done and get back the results.
	for (int x=0; x<10; x++) {
		ret=spi_device_get_trans_result(spi, &rtrans, portMAX_DELAY);
		assert(ret==ESP_OK);
		//We could inspect rtrans now if we received any info back. The LCD is treated as write-only, though.
	}
}

void send_line_finish(spi_device_handle_t spi) {
	spi_transaction_t *rtrans;
	esp_err_t ret;
	//Wait for all 6 transactions to be done and get back the results.
	for (int x=0; x<6; x++) {
		ret=spi_device_get_trans_result(spi, &rtrans, portMAX_DELAY);
		assert(ret==ESP_OK);
		//We could inspect rtrans now if we received any info back. The LCD is treated as write-only, though.
	}
}
