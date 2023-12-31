// vim:ft=cpp

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


// TEMPORARY
typedef enum SelectedColourChannel {
  SELECT_RED = 0,
  SELECT_GREEN = 1,
  SELECT_BLUE = 2
} SelectedColourChannel;

int channelValues[] = {0, 0, 0};
// END TEMPORARY



typedef struct PB_INPUT {
  bool btnL;
  bool btnR;
  RotaryEncoder::Direction rotaryDir;
  bool rotaryBtn;
} PB_INPUT;

RotaryEncoder encoder(ROT_ENC_A, ROT_ENC_B, RotaryEncoder::LatchMode::FOUR3);
PB_INPUT* inputs;
SelectedColourChannel channel;

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
  analogWriteResolution(16);
  Serial.begin(115200);

  inputs = (PB_INPUT*) malloc(sizeof(PB_INPUT));
  channel = SELECT_RED;
}

void pollInputs(PB_INPUT* inputs) {
  inputs->rotaryDir = encoder.getDirection();
  inputs->btnL = digitalRead(BTN_L);
  inputs->btnR = digitalRead(BTN_R);
  inputs->rotaryBtn = digitalRead(ROT_ENC_SW);
}

void pulseLED(int pin) {
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
  encoder.tick();
  pollInputs(inputs);
  int pin;
  if(inputs->btnL) {
    channel = SELECT_RED;
    pin = LED_R;
  } else if(inputs->rotaryBtn) {
    channel = SELECT_GREEN;
    pin = LED_G;
  } else if(inputs->btnR) {
    channel = SELECT_BLUE;
    pin = LED_B;
  }

  switch(inputs->rotaryDir) {
    case RotaryEncoder::Direction::COUNTERCLOCKWISE:
      Serial.println(channelValues[channel]);
      Serial.flush();
      analogWrite(pin, channelValues[channel]+1 > 65535 ? 65535 : ++channelValues[channel]);
      break;
    case RotaryEncoder::Direction::CLOCKWISE:
      Serial.println(channelValues[channel]);
      Serial.flush();
      analogWrite(pin, channelValues[channel]-1 < 0 ? 0 : --channelValues[channel]);
  }
}
