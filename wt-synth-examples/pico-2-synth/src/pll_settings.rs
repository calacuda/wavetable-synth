// this code is from https://github.com/AkiyukiOkayasu/pico-PDM/blob/main/src/rp2040_pll_settings_for_48khz_audio.rs
// translated with google translate.
//! RP2040_PLL_Settings for 48kHz audio
//!
//! A list of settings for running I2S PIO at 48kHz.
//! The I2S PIO runs at 15.36MHz (48kHz * 64 * 5), so the system clock should be an integer multiple of that.

use fugit::HertzU32;
use hal::pll::PLLConfig;
use rp235x_hal as hal;

/// PLL settings to run RP2040 at 76.8MHz
/// Cog is running with this setting (for low power consumption)
/// $PICO_SDK/src/rp2_common/hardware_clocks/scripts/vcocalc.py
#[allow(dead_code)]
pub const SYS_PLL_CONFIG_76P8MHZ: PLLConfig = PLLConfig {
    vco_freq: HertzU32::MHz(1536),
    refdiv: 1,
    post_div1: 5,
    post_div2: 4,
};

/// PLL settings to run RP2040 at 153.6MHz
/// $PICO_SDK/src/rp2_common/hardware_clocks/scripts/vcocalc.py
#[allow(dead_code)]
pub const SYS_PLL_CONFIG_153P6MHZ: PLLConfig = PLLConfig {
    vco_freq: HertzU32::MHz(1536),
    refdiv: 1,
    post_div1: 5,
    post_div2: 2,
};

/// PLL settings to run RP2040 at 230.4MHz
/// $PICO_SDK/src/rp2_common/hardware_clocks/scripts/vcocalc.py
#[allow(dead_code)]
pub const SYS_PLL_CONFIG_230P4MHZ: PLLConfig = PLLConfig {
    vco_freq: HertzU32::MHz(1152),
    refdiv: 1,
    post_div1: 5,
    post_div2: 1,
};

/// PLL settings to run RP2040 at 307.2MHz
/// $PICO_SDK/src/rp2_common/hardware_clocks/scripts/vcocalc.py
/// The clock may be too fast and you may need to adjust QSPI Flash.
#[allow(dead_code)]
pub const SYS_PLL_CONFIG_307P2MHZ: PLLConfig = PLLConfig {
    vco_freq: HertzU32::MHz(1536),
    refdiv: 1,
    post_div1: 5,
    post_div2: 1,
};

#[allow(dead_code)]
pub const SYS_PLL_CONFIG_300MHZ: PLLConfig = PLLConfig {
    vco_freq: HertzU32::MHz(1500),
    refdiv: 1,
    post_div1: 5,
    post_div2: 1,
};

/// PLL settings to run RP2040 at 384MHz
/// $PICO_SDK/src/rp2_common/hardware_clocks/scripts/vcocalc.py
/// The clock may be too fast and you may need to adjust QSPI Flash.
#[allow(dead_code)]
pub const SYS_PLL_CONFIG_384MHZ: PLLConfig = PLLConfig {
    vco_freq: HertzU32::MHz(1536),
    refdiv: 1,
    post_div1: 4,
    post_div2: 1,
};
