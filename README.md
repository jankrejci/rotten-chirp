# Rotten chirp
Learning project for LoRa communication using the LLCC68 based module E220-900T22D and ESP32 C3 chip.


## Image conversion
To convert PNG image to raw RGB565 format, just use the ffmpeg
```
ffmpeg -vcodec png -i image.png -vcodec rawvideo -f rawvideo -pix_fmt rgb565 image.raw
```
