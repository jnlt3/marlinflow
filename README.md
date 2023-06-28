# Marlinflow
### Neural Network training repository for the [Black Marlin](https://github.com/dsekercioglu/blackmarlin) chess engine.

This repo is or was relied upon by a number of other engines, including but not limited to [Viridithas](https://github.com/cosmobobak/viridithas), [Svart](https://github.com/crippa1337/svart), and [Carp](https://github.com/dede1751/carp).

# Requirements
- Python 3
- Cargo (Rust)
- Numpy
- PyTorch

# Usage
1. Clone the repo with `git clone https://github.com/dsekercioglu/marlinflow`
2. Build the data parser, using native CPU optimisations for your system:

```bash
cd parse
cargo rustc --release -- -C target-cpu=native
```

3. Locate the resulting `.so`/`.dll` in the `target/release/` directory and move it to the `trainer/` directory, renamed as libparse.so/libparse.dll.
4. Create some directories for training output in the `trainer/` directory:

In `trainer/`, do
```bash
mkdir nn
mkdir runs
```

5. Decide upon the directory in which you want to store your training data. (simply making a `data/` directory inside `trainer/` is a solid option)
6. Place your data file in the directory created in step 5. (if you don't have one, consult [Getting Data](#getting-data))
7. In `trainer/`, run `main.py` with the proper command line arguments:

A typical invocation for training a network looks like this:
```bash
python main.py       \
  --data-root data   \
  --train-id net0001 \
  --lr 0.001         \
  --epochs 45        \
  --lr-drop 30       \
  --batch-size 16384 \
  --wdl 0.3          \
  --scale 400        \
  --save-epochs 5
```

- `--data-root` is the directory created in step 5.
- `--train-id` is the name of the training run.
- `--lr` is the learning rate.
- `--epochs` is the number of epochs to train for.
- `--lr-drop` is the number of epochs after which the learning rate is dropped by a factor of 10.
- `--batch-size` is the batch size.
- `--wdl` is the weight of the WDL loss. (1.0 would train the network to only predict game outcome, while 0.0 would aim to predict only eval, and other values interpolate between the two)
- `--scale` is the multiplier for the sigmoid output of the final neuron.
- `--save-epochs n` tells the trainer to save the network every `n` epochs.

8. Convert the resulting JSON network file into a format usable by your engine:

The trainer will output a number of files in the `nn/` directory - files of the form `net0001_X` are saved state_dict files, which you can ignore (unless you're aiming to resume a half-completed training run) - `net0001.json` is what you're interested in: a JSON file containing the final weights of the network. 

In order to use the network, you will need to convert the JSON file into a more usable format, and you will almost certainly want to quantise it. For simple perspective networks, this can be done with [nnue-jsontobin](https://github.com/cosmobobak/nnue-jsontobin), while for more complex networks like HalfKP and HalfKA (or ones you have designed yourself!) you will need to employ some elbow grease.

# Getting Data
To train a network, you will need a large amount of training data. There are a number of possible sources for this data, the most common of which is that you will generate it using your own chess engine, which requires that you write some datagen code. It is recommended that your data generator produce data directly in the marlinflow data format, and not in the legacy text format (see [Legacy Text Format](#legacy-text-format)), as it is a significantly more compact format, and skips the required conversion step.

To convert a file in the legacy text format into a data file, use marlinflow-utils, which is built in much the same way as the parser:
```bash
cd utils
cargo rustc --release -- -C target-cpu=native
```
The resulting binary will be in `target/release/`, and can be invoked as follows:
```bash
target/release/marlinflow-utils txt-to-data INPUT.txt --output OUTPUT.bin
```

# Legacy Text Format
Marlinflow accepts a specific text format for conversion into data files, with lines set out as following:
```
<fen0> | <eval0> | <wdl0>
<fen1> | <eval1> | <wdl1>
```
Here, `<fen>` is a [FEN string](https://www.chessprogramming.org/Forsyth-Edwards_Notation), `<eval>` is a evaluation in centipawns from white's point of view, and `<wdl>` is 1.0, 0.5, or 0.0, representing a win for white, a draw, or a win for black, respectively.

# Marlinflow-Utils
`marlinflow-utils` is a program that provides a number of utilities for working with marlinflow. These are as follows:
- `txt-to-data` converts a legacy text file into a data file.
- `shuffle` shuffles a data file. It is extremely important to shuffle your data before training, to prevent overfitting.
- `interleave` randomly interleaves data files. This allows you to cleanly combine data from multiple sources without requiring a re-shuffle, provided that the source files have already been shuffled.
- `convert` will convert an NNUE JSON file into the BlackMarlin NNUE format. (currently only supports HalfKP)