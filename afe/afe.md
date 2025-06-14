# We need to select an ADC.
We already have a preamp based on the INA128 and some sallen-key filters, now we need to sample them with an ADC.
Alternatively, we could buy an entire analog frontend chip outright. 
What chips are available?
Part,Supply Voltage,Resolution (bits),Channels,Sample Rate,Type,Application,Interface,Price (1k),Package,Input Type,CMRR,Input Range,Sampling Type
ADS1298,3.3V,24,8,8kSPS,Full AFE,Medical,SPI,$18.50,TQFP-64,Differential,115dB,±4V,Multiplexed
ADS1220,2.3V-5.5V,24,4,2kSPS,ADC,Industrial,SPI,$6.20,TSSOP-16,Differential,110dB,±VREF,Multiplexed
ADS1299,3.3V,24,8,16kSPS,Full AFE,Medical (EEG),SPI,$42.00,TQFP-64,Differential,120dB,±4V,Multiplexed
ADAS1000,1.8V/3.3V,24,5,128kSPS,Full AFE,Medical (ECG),SPI,$15.80,LFCSP-40,Differential,110dB,±2.4V,Multiplexed
AD8232,2.0V-3.5V,N/A,1,N/A,Full AFE,Medical (ECG),Analog,$2.95,LFCSP-20,Differential,80dB,±0.3V,N/A
MAX30001,1.8V/3.3V,18,1,128kSPS,Full AFE,Medical (ECG),SPI,$9.90,WLP-25,Differential,100dB,±2.4V,N/A
AD7768,1.8V/3.3V,24,8,256kSPS,ADC,Industrial,SPI,$28.50,LFCSP-40,Differential,105dB,±VREF,Simultaneous
ADS1262,2.3V-5.5V,32,2,38kSPS,ADC,Industrial,SPI,$12.40,TSSOP-28,Differential,115dB,±VREF,Multiplexed
LTC2983,2.85V-5.25V,24,20,N/A,ADC,Industrial,SPI,$22.60,LQFP-48,Single-ended,N/A,0-VREF,Multiplexed
