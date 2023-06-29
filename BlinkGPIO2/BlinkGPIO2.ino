#include <arduino.h>
#include <RotaryEncoder.h>

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

RotaryEncoder encoder(ROT_ENC_A, ROT_ENC_B, RotaryEncoder::LatchMode::TWO03);

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

void loop() {
  encoder.tick();
  RotaryEncoder::Direction spinDir = encoder.getDirection();
  unsigned long spd = encoder.getRPM();
  switch(spinDir) {
    case RotaryEncoder::CLOCKWISE:
      Serial.print("rotating clockwise! ");
      break;
    case RotaryEncoder::COUNTERCLOCKWISE:
      Serial.print("rotating counterclockwise! ");
      break;
    default:
      Serial.print("not rotating! ");
  }
  Serial.println(spd);
}
