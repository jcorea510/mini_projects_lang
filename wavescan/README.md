# wavescan
An simple waveform analyzer. It just plot 
time domain signal of wave file and its frecuency domain representation
in an image.

My interest of this simple program is to understand 
clap (an argument parse library) and plotters (an graph drawer library).

## Dependecies
wavers

plotters

rustfft

clap

## How to use
Run the following commands. The output will be in an png in resources folder.
You can also specify an file output adding the command --output

```zsh
cargo build

./target/debug/wavescan --input ./resources/fubuki.wav
```

![](./resources/fubuki-noise.png)
