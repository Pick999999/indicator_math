// ============================================================
// lib.rs — Full Technical Indicators + EMA Analysis
// ============================================================

#[derive(Debug, Clone, Copy)]
pub struct Candle {
    pub time: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct ValueAtTime {
    pub time: u64,
    pub value: f64,
}

// ============================================================
// Helper Functions
// ============================================================

fn extract_close(candles: &[Candle]) -> Vec<f64> {
    candles.iter().map(|c| c.close).collect()
}

fn wrap_output(candles: &[Candle], values: Vec<f64>) -> Vec<ValueAtTime> {
    candles.iter().zip(values.iter())
        .map(|(c, v)| ValueAtTime { time: c.time, value: *v })
        .collect()
}

// ============================================================
// SMA
// ============================================================

pub fn sma(candles: &[Candle], period: usize) -> Vec<ValueAtTime> {
    let prices = extract_close(candles);
    let mut out = vec![f64::NAN; prices.len()];

    if period == 0 || prices.len() < period {
        return wrap_output(candles, out);
    }

    for i in period - 1..prices.len() {
        let sum: f64 = prices[i - period + 1..=i].iter().sum();
        out[i] = sum / period as f64;
    }

    wrap_output(candles, out)
}

// ============================================================
// EMA
// ============================================================

pub fn ema(candles: &[Candle], period: usize) -> Vec<ValueAtTime> {
    let prices = extract_close(candles);
    let mut out = vec![f64::NAN; prices.len()];

    if period == 0 || prices.is_empty() {
        return wrap_output(candles, out);
    }

    let k = 2.0 / (period as f64 + 1.0);
    let mut prev = prices[0];

    for i in 0..prices.len() {
        if i < period - 1 {
            out[i] = f64::NAN;
        } else if i == period - 1 {
            let sma_val: f64 = prices[0..period].iter().sum::<f64>() / period as f64;
            out[i] = sma_val;
            prev = sma_val;
        } else {
            prev = prices[i] * k + prev * (1.0 - k);
            out[i] = prev;
        }
    }

    wrap_output(candles, out)
}

// ============================================================
// WMA
// ============================================================

pub fn wma(candles: &[Candle], period: usize) -> Vec<ValueAtTime> {
    let prices = extract_close(candles);
    let mut out = vec![f64::NAN; prices.len()];
    let denom = (period * (period + 1) / 2) as f64;

    for i in period - 1..prices.len() {
        let mut sum = 0.0;
        for j in 0..period {
            sum += prices[i - j] * (period - j) as f64;
        }
        out[i] = sum / denom;
    }

    wrap_output(candles, out)
}

fn wma_values(values: &[f64], period: usize) -> Vec<f64> {
    let mut out = vec![f64::NAN; values.len()];
    if period == 0 || values.len() < period {
        return out;
    }

    let denom = (period * (period + 1) / 2) as f64;

    for i in period - 1..values.len() {
        let mut sum = 0.0;
        for j in 0..period {
            sum += values[i - j] * (period - j) as f64;
        }
        out[i] = sum / denom;
    }

    out
}

// ============================================================
// HMA
// ============================================================

pub fn hma(candles: &[Candle], period: usize) -> Vec<ValueAtTime> {
    if period < 2 {
        return wrap_output(candles, vec![f64::NAN; candles.len()]);
    }

    let prices = extract_close(candles);
    let half = period / 2;
    let sqrt_n = (period as f64).sqrt().round() as usize;

    let w1 = wma_values(&prices, half);
    let w2 = wma_values(&prices, period);

    let diff: Vec<f64> = w1.iter().zip(w2.iter()).map(|(a, b)| 2.0 * a - b).collect();
    let h = wma_values(&diff, sqrt_n);

    wrap_output(candles, h)
}

// ============================================================
// EHMA = HMA of EMA
// ============================================================

pub fn ehma(candles: &[Candle], period: usize) -> Vec<ValueAtTime> {
    let ema_vals: Vec<f64> = ema(candles, period).iter().map(|x| x.value).collect();

    let temp: Vec<Candle> = candles.iter().zip(ema_vals.iter())
        .map(|(c, v)| Candle { time: c.time, open: c.open, high: c.high, low: c.low, close: *v })
        .collect();

    hma(&temp, period)
}

// ============================================================
// MACD
// ============================================================

pub struct MacdResult {
    pub macd: Vec<ValueAtTime>,
    pub signal: Vec<ValueAtTime>,
    pub histogram: Vec<ValueAtTime>,
}

pub fn macd(candles: &[Candle], fast: usize, slow: usize, signal_p: usize) -> MacdResult {
    let ema_fast: Vec<f64> = ema(candles, fast).iter().map(|v| v.value).collect();
    let ema_slow: Vec<f64> = ema(candles, slow).iter().map(|v| v.value).collect();

    let macd_line: Vec<f64> = ema_fast.iter().zip(ema_slow.iter()).map(|(f, s)| f - s).collect();

    let temp: Vec<Candle> = candles.iter().zip(macd_line.iter())
        .map(|(c, v)| Candle { time: c.time, open: c.open, high: c.high, low: c.low, close: *v })
        .collect();

    let signal_line: Vec<f64> = ema(&temp, signal_p).iter().map(|v| v.value).collect();
    let hist: Vec<f64> = macd_line.iter().zip(signal_line.iter()).map(|(m, s)| m - s).collect();

    MacdResult {
        macd: wrap_output(candles, macd_line),
        signal: wrap_output(candles, signal_line),
        histogram: wrap_output(candles, hist),
    }
}

// ============================================================
// EMA Analysis
// ============================================================

fn slope(v1: f64, v2: f64) -> f64 {
    v2 - v1
}

fn slope_direction(v: f64) -> &'static str {
    if v > 0.0 { "Up" }
    else if v < 0.0 { "Down" }
    else { "Parallel" }
}

fn turn_type(prev: f64, now: f64, next: f64) -> &'static str {
    if prev > now && next > now { "TurnUp" }
    else if prev < now && next < now { "TurnDown" }
    else { "None" }
}

fn ema_position(c: &Candle, v: f64) -> &'static str {
    if v > c.high { "AboveCandle" }
    else if v < c.low { "BelowCandle" }
    else { "InsideCandle" }
}

fn ema_above(short: f64, long: f64) -> &'static str {
    if short > long { "ShortAbove" }
    else if short < long { "LongAbove" }
    else { "Equal" }
}

fn back_turn(list: &[&'static str], idx: usize, back: usize) -> &'static str {
    if idx >= back { list[idx - back] } else { "None" }
}

// ============================================================
// Output Struct ของ Analysis
// ============================================================

pub struct EmaAnalysis {
    pub time_candle: u64,
    pub color_candle: &'static str,

    pub ema_short_value: f64,
    pub ema_short_slope_value: f64,
    pub ema_short_slope_direction: &'static str,
    pub is_ema_short_turn_type: &'static str,
    pub short_distance_from_last_turn: usize,
    pub position_short: &'static str,

    pub ema_long_value: f64,
    pub ema_long_slope_value: f64,
    pub ema_long_slope_direction: &'static str,
    pub is_ema_long_turn_type: &'static str,
    pub long_distance_from_last_turn: usize,
    pub position_long: &'static str,

    pub is_ema_cut_type: &'static str,
    pub distance_from_cut_point: usize,

    pub previous_color_back1: &'static str,
    pub previous_color_back3: &'static str,

    pub is_ema_short_turn_type_back1: &'static str,
    pub is_ema_short_turn_type_back2: &'static str,
    pub is_ema_short_turn_type_back3: &'static str,
    pub is_ema_short_turn_type_back4: &'static str,

    pub ema_above: &'static str,
    pub ema_above_diff: f64,
}

// ============================================================
// Main Analysis Function
// ============================================================

pub fn analyze_ema(candles: &[Candle], short_p: usize, long_p: usize) -> Vec<EmaAnalysis> {
    let ema_short = ema(candles, short_p);
    let ema_long = ema(candles, long_p);

    let mut out = vec![];

    let mut last_turn_short = 0;
    let mut last_turn_long = 0;
    let mut last_cut = 0;

    let mut short_turn_list = vec!["None"; candles.len()];

    for i in 2..candles.len() - 1 {
        let c = &candles[i];

        // slopes
        let slope_short = slope(ema_short[i - 1].value, ema_short[i].value);
        let slope_long = slope(ema_long[i - 1].value, ema_long[i].value);

        // turn type short
        let turn_s = turn_type(
            ema_short[i - 1].value,
            ema_short[i].value,
            ema_short[i + 1].value,
        );

        short_turn_list[i] = turn_s;

        if turn_s != "None" {
            last_turn_short = i;
        }

        // turn long
        let turn_l = turn_type(
            ema_long[i - 1].value,
            ema_long[i].value,
            ema_long[i + 1].value,
        );

        if turn_l != "None" {
            last_turn_long = i;
        }

        // ema cut
        let cut = if ema_short[i - 1].value < ema_long[i - 1].value &&
                     ema_short[i].value > ema_long[i].value {
            "GoldenCross"
        } else if ema_short[i - 1].value > ema_long[i - 1].value &&
                    ema_short[i].value < ema_long[i].value {
            "DeathCross"
        } else {
            "None"
        };

        if cut != "None" {
            last_cut = i;
        }

        out.push(EmaAnalysis {
            time_candle: c.time,
            color_candle: if c.close > c.open { "Green" }
                          else if c.close < c.open { "Red" }
                          else { "Equal" },

            ema_short_value: ema_short[i].value,
            ema_short_slope_value: slope_short,
            ema_short_slope_direction: slope_direction(slope_short),
            is_ema_short_turn_type: turn_s,
            short_distance_from_last_turn: i - last_turn_short,
            position_short: ema_position(c, ema_short[i].value),

            ema_long_value: ema_long[i].value,
            ema_long_slope_value: slope_long,
            ema_long_slope_direction: slope_direction(slope_long),
            is_ema_long_turn_type: turn_l,
            long_distance_from_last_turn: i - last_turn_long,
            position_long: ema_position(c, ema_long[i].value),

            is_ema_cut_type: cut,
            distance_from_cut_point: i - last_cut,

            previous_color_back1:
                if candles[i - 1].close > candles[i - 1].open { "Green" }
                else if candles[i - 1].close < candles[i - 1].open { "Red" }
                else { "Equal" },

            previous_color_back3:
                if i >= 3 {
                    if candles[i - 3].close > candles[i - 3].open { "Green" }
                    else if candles[i - 3].close < candles[i - 3].open { "Red" }
                    else { "Equal" }
                } else {
                    "Unknown"
                },

            is_ema_short_turn_type_back1: back_turn(&short_turn_list, i, 1),
            is_ema_short_turn_type_back2: back_turn(&short_turn_list, i, 2),
            is_ema_short_turn_type_back3: back_turn(&short_turn_list, i, 3),
            is_ema_short_turn_type_back4: back_turn(&short_turn_list, i, 4),

            ema_above: ema_above(ema_short[i].value, ema_long[i].value),
            ema_above_diff: ema_short[i].value - ema_long[i].value,
        });
    }

    out
}
