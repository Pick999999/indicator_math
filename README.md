# indicator_math Ver 0.6.6

# Technical Indicators EMA

A Rust library for technical analysis with EMA (Exponential Moving Average) and various moving averages.

## Features

- Multiple Moving Averages: SMA, EMA, WMA, HMA, EHMA
- MACD indicator
- Comprehensive EMA analysis with turning points
- Trading signal generation (Call/Put/Hold)

## Installation

Add this to your `Cargo.toml`:

\`\`\`toml
[dependencies]
technical-indicators-ema = "0.1.0"
\`\`\`

## Usage

\`\`\`rust
use technical_indicators_ema::{Candle, analyze_ema, MaType, get_action_by_simple};

let candles = vec![
    Candle { time: 1, open: 100.0, high: 105.0, low: 99.0, close: 103.0 },
    // ... more candles
];

// Analyze with EMA
let analysis = analyze_ema(&candles, 9, 21, MaType::EMA);

// Get trading signal
for a in &analysis {
    let action = get_action_by_simple(a);
    println!("Action: {:?}", action);
}
\`\`\`

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

your-project/
├── Cargo.toml
├── README.md
├── LICENSE-MIT (หรือ LICENSE)
├── LICENSE-APACHE (ถ้าใช้ dual license)
└── src/
    └── lib.rs