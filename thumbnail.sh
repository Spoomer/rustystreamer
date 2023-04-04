#!/bin/bash

# Parameter: 1: quantity 2: path to video 3: outputpath exluding file extension
duration=$(ffprobe -v error -show_entries format=duration -of default=noprint_wrappers=1:nokey=1 $2)

duration=$(LC_NUMERIC="en_US.UTF-8" printf "%.0f\n" "$duration")
quantity=$1
multipli=$(($duration / $quantity))
while [ $multipli == 0 ]
do
  quantity=$(($quantity-($quantity/2)))
  echo $quantity
  if (($quantity<=1));
  then
    multipli=1
    break
  else
    multipli=$(($duration / $quantity))
  fi 
done
seq=$(($quantity-1));
for i in `seq 0 $seq`
do
  t=$(($i*$multipli))
  ffmpeg -hide_banner -loglevel error -ss $t -i $2 -frames:v 1 -s 320x240 $3_$t.jpg
done
