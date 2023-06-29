#include <arduino.h>
#include "RotaryEncoder.h"

#define SPI_MOSI 5      // Master Out Slave In
#define SPI_MISO 17     // Master In Slave Out
#define SPI_CS_LCD 18
#define SPI_CLK 19
#define SPI_CS_PSRAM 21
#define I2C_SDA 22
#define I2C_SCL 23
#define PWM0 25
#define PWM1 26
#define PWM2 27
#define PWM3 14
#define BTN_L 0
#define BTN_R 33
#define ROT_ENC_SW 34
#define ROT_ENC_A 35
#define ROT_ENC_B 32
#define LCD_BL 12
#define LED_R 13
#define LED_G 15
#define LED_B 2

typedef struct PB_INPUT {
  bool btnL;
  bool btnR;
  RotaryEncoder::Direction rotaryDir;
  bool rotaryBtn;
} PB_INPUT;

RotaryEncoder encoder(ROT_ENC_A, ROT_ENC_B, RotaryEncoder::LatchMode::FOUR3);

void setup() {
  pinMode(SPI_MOSI, OUTPUT);
  pinMode(SPI_MISO, INPUT);
  pinMode(SPI_CS_LCD, OUTPUT);
  pinMode(SPI_CLK, OUTPUT); 
  pinMode(SPI_CS_PSRAM, OUTPUT);
  pinMode(PWM0, OUTPUT);
  pinMode(PWM1, OUTPUT);
  pinMode(PWM2, OUTPUT);
  pinMode(PWM3, OUTPUT);
  pinMode(BTN_L, INPUT);
  pinMode(BTN_R, INPUT);
  pinMode(ROT_ENC_SW, INPUT);
  pinMode(ROT_ENC_A, INPUT);
  pinMode(ROT_ENC_B, INPUT);
  pinMode(LCD_BL, OUTPUT);
  pinMode(LED_R, OUTPUT);
  pinMode(LED_G, OUTPUT);
  pinMode(LED_B, OUTPUT);
  Serial.begin(115200);
}

PB_INPUT* pollInputs(RotaryEncoder encoder) {
  PB_INPUT* inputs = (PB_INPUT*) malloc(sizeof(PB_INPUT));
  inputs->rotaryDir = encoder.getDirection();
  inputs->btnL = digitalRead(BTN_L);
  inputs->btnR = digitalRead(BTN_R);
  inputs->rotaryBtn = digitalRead(ROT_ENC_SW);
  return inputs;
}

void pulse(int pin) {
  for(int i=0;i<256;++i) {
    analogWrite(pin, i);
    delay(5);
  }
  for(int i=255;i>=0;--i) {
    analogWrite(pin, i);
    delay(5);
  }
}

void loop() {
  /* encoder.tick(); */
  /* PB_INPUT* inputs = pollInputs(encoder); */
  /* switch(inputs->rotaryDir) { */
  /*   case RotaryEncoder::Direction::CLOCKWISE: */
  /*     Serial.println("rotating clockwise! "); */
  /*     break; */
  /*   case RotaryEncoder::Direction::COUNTERCLOCKWISE: */
  /*     Serial.println("rotating counterclockwise! "); */
  /*     break; */
  /*   default: */
  /*     break; */
  /* } */

  pulse(LED_R);
  pulse(LED_G);
  pulse(LED_B);
}
