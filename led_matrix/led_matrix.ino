#include <FastLED.h>
#include <assert.h>

#include <array>
#include <tuple>

// Hardware configuration.
//
// There are two sections of LEDs. Both sections have the same number of rows
// and columns. When referring to both sections we use FULL_*, and when
// referring to just one section we use HALF_*.
#define NUM_ROWS_HALF 6
#define NUM_ROWS_FULL 2 * NUM_ROWS_HALF
#define NUM_COLS 56
#define NUM_LEDS_FULL (NUM_ROWS_FULL * NUM_COLS)
#define NUM_LEDS_HALF (NUM_ROWS_HALF * NUM_COLS)
#define LED_TYPE LPD8806
#define DATA_PIN_TOP 4
#define CLOCK_PIN_TOP 2
#define DATA_PIN_BOT 8
#define CLOCK_PIN_BOT 6

// Software configuration.
#define FPS 60             // Frames per second.
#define MSPF (1000 / FPS)  // Milliseconds per frame.
#define TWO_PI 2.0f * PI

// Define the two sections of LEDs. One top and one bottom.
CRGB leds_top[NUM_LEDS_HALF];
CRGB leds_bot[NUM_LEDS_HALF];

enum Mode { MODE_OFF, MODE_OCTOPUS_SKIN, MODE_RUNNING_BLOCKS };
Mode current_mode = MODE_OFF;
Mode previous_mode = MODE_OFF;

// Returns `val` wrapped to between 0 and `max` - 1.
int wrap(int val, int max) {
  while (val < 0) val += max;
  while (val >= max) val -= max;
  return val;
}

// Returns a 2D tuple (row index, column index) from a 1D LED index.
std::tuple<int, int> idx_2d_from_1d(int led_idx) {
  assert(led_idx >= 0 && led_idx < NUM_LEDS_FULL);

  int row = led_idx / NUM_COLS;
  int col = led_idx % NUM_COLS;
  if (row % 2 != 0) col = NUM_COLS - 1 - col;
  return {row, col};
}

// Given an LED index between 0 and NUM_LEDS_FULL - 1, returns a mutable
// reference to the LED data that allows for querying and modifying the LED
// color.
CRGB& get_led(int led_idx) {
  assert(led_idx >= 0 && led_idx < NUM_LEDS_FULL);

  if (led_idx < NUM_LEDS_HALF) return leds_top[led_idx];
  return leds_bot[led_idx - NUM_LEDS_HALF];
}

// Sine and cosine functions that output in [0.0, 1.0] instead of [-1.0, 1.0].
float unit_sin(float x) { return (sin(x) + 1.0f) / 2.0f; }
float unit_cos(float x) { return (cos(x) + 1.0f) / 2.0f; }

void octopus_skin() {
  static constexpr uint32_t CYCLE_TIME_MS = 15000;  // In milliseconds.

  // Precompute necessary float conversions.
  static constexpr float cycle_time_ms_f = CYCLE_TIME_MS;
  static constexpr float num_idxs_per_cycle = 18.0f;

  // A full cycle starts with time_since_cycle_start_ms at cycle_start_time_ms
  // and goes until time_since_cycle_start_ms hits CYCLE_TIME_MS.
  static uint32_t cycle_start_time_ms = millis();
  static uint32_t last_frame_time_ms = 0;

  uint32_t now_ms = millis();
  // If it's not time for a new frame yet, then return.
  if (now_ms - last_frame_time_ms < MSPF) return;
  // Time for a new frame. Update the last frame time.
  last_frame_time_ms = now_ms;
  // Compute how far we are into the current display cycle.
  uint32_t time_since_cycle_start_ms = now_ms - cycle_start_time_ms;

  // Fade the brightness of the whole display in and out to look sortof like
  // fluttering breathing.
  //
  // Don't go all the way to 255 to avoid washing out the colors. It gets really
  // blue near 255.
  static constexpr uint8_t MAX_VALUE = 175;
  static constexpr uint8_t MIN_VALUE = 25;
  const float time_frac = time_since_cycle_start_ms / cycle_time_ms_f;
  const uint8_t value =
      MIN_VALUE +
      (MAX_VALUE - MIN_VALUE) * (0.5f * unit_sin(2.0f * TWO_PI * time_frac) +
                                 0.5f * unit_sin(8.0f * TWO_PI * time_frac));

  // Make the patterns go in a circular-ish motion by offsetting the row and
  // column using sine and cosine.
  static constexpr int MAX_ROW_TRAVEL_DIST = 24;
  static constexpr int MAX_COL_TRAVEL_DIST = 48;
  const int row_idx_offset =
      MAX_ROW_TRAVEL_DIST *
      unit_sin(TWO_PI * time_since_cycle_start_ms / cycle_time_ms_f);
  const int col_idx_offset =
      MAX_COL_TRAVEL_DIST *
      unit_cos(TWO_PI * time_since_cycle_start_ms / cycle_time_ms_f);

  for (int led_idx = 0; led_idx < NUM_LEDS_FULL; ++led_idx) {
    // Get the 2D position (row and column) for this LED.
    auto [row_idx, col_idx] = idx_2d_from_1d(led_idx);
    // Apply the row/col offsets and apply wrapping.
    row_idx = wrap(row_idx + row_idx_offset, NUM_ROWS_FULL);
    col_idx = wrap(col_idx + col_idx_offset, NUM_COLS);
    // Compute the hue based on the row and column indices.
    const uint8_t hue =
        wrap(80.f + 127.5f * (unit_sin(TWO_PI * row_idx / num_idxs_per_cycle) +
                              unit_sin(TWO_PI * col_idx / num_idxs_per_cycle)),
             256);
    // Set the LED color.
    get_led(led_idx) = CHSV(hue, 255, value);
  }
  FastLED.show();

  // If we have gone through a full display cycle, then start over with a new
  // cycle.
  if (time_since_cycle_start_ms >= CYCLE_TIME_MS) {
    cycle_start_time_ms = now_ms;
  }
}

int get_hue_dist(uint8_t hue1, uint8_t hue2) {
  int dist = abs(static_cast<int>(hue1) - static_cast<int>(hue2));
  return std::min(dist, 256 - dist);
}

void running_blocks() {
  static constexpr int BLOCK_SIZE = 8;
  static constexpr int NUM_BLOCKS = NUM_COLS / BLOCK_SIZE + 2;
  static constexpr size_t NUM_BLOCKS_SIZE_T = NUM_BLOCKS;
  static constexpr int MIN_ADJACENT_HUE_DIST = 50;
  static constexpr uint32_t MS_PER_SHIFT = 80;

  static uint32_t last_frame_time_ms = 0;
  uint32_t now_ms = millis();
  // If it's not time for a new frame yet, then return.
  if (now_ms - last_frame_time_ms < MS_PER_SHIFT) return;
  // Time for a new frame. Update the last frame time.
  last_frame_time_ms = now_ms;

  auto get_random_hue = []() -> uint8_t { return random(0, 256); };

  // Use fixed-size arrays instead of vectors. Maintain a current count and
  // simple helpers to fill and erase the front element (shift-left).
  using BlockArray = std::array<uint8_t, NUM_BLOCKS_SIZE_T>;

  auto get_new_block_hue = [&](const BlockArray& block_hues,
                               size_t count) -> uint8_t {
    uint8_t hue = get_random_hue();
    if (count > 0) {
      const uint8_t adjacent_hue = block_hues[count - 1];
      while (get_hue_dist(hue, adjacent_hue) < MIN_ADJACENT_HUE_DIST) {
        hue = get_random_hue();
      }
    }
    return hue;
  };

  auto fill_block_hues = [&](BlockArray& block_hues, size_t& count) {
    while (count < NUM_BLOCKS_SIZE_T) {
      block_hues[count++] = get_new_block_hue(block_hues, count);
    }
  };

  auto render = [](CRGB* leds, const BlockArray& block_hues, int block_shift) {
    for (int led_idx = 0; led_idx < NUM_LEDS_HALF; ++led_idx) {
      const auto [row_idx, col_idx] = idx_2d_from_1d(led_idx);
      const int block_idx = (col_idx + block_shift) / BLOCK_SIZE;
      const uint8_t hue = block_hues[block_idx];
      leds[led_idx] = CHSV(hue, 255, 255);
    }
  };

  auto erase_front = [](BlockArray& block_hues, size_t& count) {
    if (count == 0) return;
    for (size_t i = 0; i + 1 < count; ++i) block_hues[i] = block_hues[i + 1];
    if (count > 0) --count;
  };

  static int block_shift = 0;  // Number of columns shifted.
  static std::array<uint8_t, NUM_BLOCKS_SIZE_T> block_hues_top;
  static std::array<uint8_t, NUM_BLOCKS_SIZE_T> block_hues_bot;
  static size_t block_hues_top_count = 0;
  static size_t block_hues_bot_count = 0;

  // Shift the blocks by one column. If a block has fully shifted off, pop it
  // and add a new one.
  ++block_shift;
  if (block_shift >= BLOCK_SIZE) {
    block_shift = 0;
    erase_front(block_hues_top, block_hues_top_count);
    erase_front(block_hues_bot, block_hues_bot_count);
  }

  fill_block_hues(block_hues_top, block_hues_top_count);
  fill_block_hues(block_hues_bot, block_hues_bot_count);

  render(leds_top, block_hues_top, block_shift);
  render(leds_bot, block_hues_bot, block_shift);
  FastLED.show();
}

void color_sampler() {
  std::vector<uint8_t> hues = {0,   10,  20,  30,  40,  50,  60,  70,  80,
                               90,  100, 110, 120, 130, 140, 150, 160, 170,
                               180, 190, 200, 210, 220, 230, 240, 250};

  // Draw 6 columns of solid color across the entire display.
  int column_width = NUM_COLS / hues.size();
  int hue_idx = 0;
  for (int led_idx = 0; led_idx < NUM_LEDS_FULL; ++led_idx) {
    auto [row_idx, col_idx] = idx_2d_from_1d(led_idx);
    if (hue_idx >= hues.size() || col_idx % 2 == 0) continue;
    const uint8_t hue = hues[hue_idx++];
    get_led(led_idx) = CHSV(hue, 255, 255);
  }
  FastLED.show();
}

void set_mode_from_serial() {
  if (!Serial.available()) return;

  String mode_str = Serial.readStringUntil('\n');
  mode_str.trim();
  mode_str.toUpperCase();

  if (mode_str == "OCTOPUS_SKIN") {
    current_mode = MODE_OCTOPUS_SKIN;
    Serial.println("MODE: OCTOPUS SKIN");
  } else if (mode_str == "BLOCKS") {
    current_mode = MODE_RUNNING_BLOCKS;
    Serial.println("MODE: RUNNING BLOCKS");
  } else if (mode_str == "OFF") {
    current_mode = MODE_OFF;
    Serial.println("MODE: OFF");
  } else {
    Serial.print("UNKNOWN MODE: ");
    Serial.println(mode_str);
  }
}

void setup() {
  Serial.begin(9600);

  FastLED.addLeds<LPD8806, DATA_PIN_TOP, CLOCK_PIN_TOP, BRG>(leds_top,
                                                             NUM_LEDS_HALF);
  FastLED.addLeds<LPD8806, DATA_PIN_BOT, CLOCK_PIN_BOT, BRG>(leds_bot,
                                                             NUM_LEDS_HALF);
  FastLED.setBrightness(255);
  FastLED.clear();
  FastLED.show();
}

void loop() {
  set_mode_from_serial();

  switch (current_mode) {
    case MODE_OCTOPUS_SKIN:
      octopus_skin();
      break;
    case MODE_RUNNING_BLOCKS:
      running_blocks();
      break;
    case MODE_OFF:
      if (previous_mode != MODE_OFF) {
        // Just switched to OFF mode. Clear the display.
        FastLED.clear();
        FastLED.show();
      }
      delay(100);
      break;
    default:
      // Do nothing.
      delay(100);
      break;
  }
  previous_mode = current_mode;
}
