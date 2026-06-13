//! ESP32-C3 SuperMini 引脚定义（可按实际接线修改）

pub mod config;
pub mod led;
pub mod pins;
pub mod status_led;

/// 板载 LED（多数 SuperMini 克隆为 GPIO8，低电平点亮）
pub const LED_GPIO: u8 = 8;

/// 霍尔传感器数字输入（KY-003 的 S 脚，靠近磁铁时拉低/拉高取决于模块）
pub const HALL_GPIO: u8 = 4;

/// SSD1306 I2C
pub const I2C_SDA_GPIO: u8 = 5;
pub const I2C_SCL_GPIO: u8 = 6;
pub const OLED_I2C_ADDR: u8 = 0x3C;
