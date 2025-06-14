/*
## pins
ADS131M08 -> Teensy4.1
SYNC -> 2
DRDY -> 9
CS -> 10 (CS)       | LPSPI4
DIN -> 11 (MOSI)    |
DOUT -> 12 (MISO)   |
SCLK -> 13 (SCK)    |
CLKIN -> 23

I made a silly mistake in PCB design and pin 13 is connected to 27
You have to leave 27 floating

*/

#include <SPI.h>

// Clock configuration
#define CLOCK_PIN 23  // CLKIN pin
#define CLOCK_FREQ 8000000  // 8MHz

// SPI configuration
#define CS_PIN 10
#define DRDY_PIN 9

// Buffer for SPI data
uint32_t spiBuffer[10];

void setup() {
  Serial.begin(115200);  // Initialize serial communication at 115200 baud
  Serial.println("up");  // Print "up" once at startup

  // Configure FlexPWM for clock generation
  // Using FlexPWM1_0 for pin 23
  FLEXPWM1_MCTRL |= FLEXPWM_MCTRL_CLDOK(0x0F);  // Load OK for all submodules
  FLEXPWM1_SM0CTRL2 = FLEXPWM_SMCTRL2_INDEP;    // Independent mode
  FLEXPWM1_SM0CTRL = FLEXPWM_SMCTRL_FULL;       // Full cycle reload
  FLEXPWM1_SM0INIT = 0;                         // Initial count
  FLEXPWM1_SM0VAL0 = 0;                         // Value for 0% duty cycle
  FLEXPWM1_SM0VAL1 = (F_CPU / (2 * CLOCK_FREQ)) - 1;  // Value for 50% duty cycle
  FLEXPWM1_SM0VAL2 = 0;                         // Value for 0% duty cycle
  FLEXPWM1_SM0VAL3 = 0;                         // Value for 0% duty cycle
  FLEXPWM1_SM0VAL4 = 0;                         // Value for 0% duty cycle
  FLEXPWM1_SM0VAL5 = 0;                         // Value for 0% duty cycle
  
  // Configure pin 23 for FlexPWM output
  IOMUXC_SW_MUX_CTL_PAD_GPIO_EMC_23 = 1;  // ALT1 for FlexPWM
  IOMUXC_SW_PAD_CTL_PAD_GPIO_EMC_23 = IOMUXC_PAD_DSE(7) | IOMUXC_PAD_SPEED(3);
  
  // Enable FlexPWM1_0
  FLEXPWM1_MCTRL |= FLEXPWM_MCTRL_RUN(0x0F);  // Run all submodules

  // Configure SPI pins
  pinMode(CS_PIN, OUTPUT);
  pinMode(DRDY_PIN, INPUT);
  
  // Initialize SPI
  SPI.begin();
  SPI.beginTransaction(SPISettings(8000000, MSBFIRST, SPI_MODE1));
  
  // Set initial pin states
  digitalWrite(CS_PIN, HIGH);
}

void loop() {
  // Wait for DRDY to go low (data ready)
  Serial.println("waiting for drdy low...");
  while (digitalRead(DRDY_PIN) != LOW) {}
   
  digitalWrite(CS_PIN, LOW);  // Assert CS
  
  // Read 10 32-bit words
  for (int i = 0; i < 10; i++) {
    spiBuffer[i] = 0;
    // Read 4 bytes to form 32-bit word
    for (int j = 0; j < 4; j++) {
      spiBuffer[i] = (spiBuffer[i] << 8) | SPI.transfer(0x00);
    }
  }
  
  digitalWrite(CS_PIN, HIGH);  // Deassert CS
  
  // Print the received data
  Serial.println("Received data:");
  for (int i = 0; i < 10; i++) {
    Serial.print("Word ");
    Serial.print(i);
    Serial.print(": 0x");
    Serial.println(spiBuffer[i], HEX);
  }
  Serial.println();
  
}
