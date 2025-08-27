#!/bin/bash

mkdir -p notifications
mkdir -p background

generate_tone() {
    local filename=$1
    local frequency=$2
    local duration=$3
    local fade_in=${4:-0.1}
    local fade_out=${5:-0.1}
    
    ffmpeg -f lavfi -i "sine=frequency=${frequency}:duration=${duration}" \
           -af "afade=t=in:st=0:d=${fade_in},afade=t=out:st=${duration}:d=${fade_out}" \
           -ar 44100 -ac 2 -b:a 128k \
           "${filename}" -y 2>/dev/null
}

generate_noise() {
    local filename=$1
    local noise_type=$2
    local duration=$3
    
    case $noise_type in
        white)
            ffmpeg -f lavfi -i "anoisesrc=duration=${duration}:color=white:amplitude=0.1" \
                   -ar 44100 -ac 2 -b:a 128k \
                   "${filename}" -y 2>/dev/null
            ;;
        brown)
            ffmpeg -f lavfi -i "anoisesrc=duration=${duration}:color=brown:amplitude=0.1" \
                   -ar 44100 -ac 2 -b:a 128k \
                   "${filename}" -y 2>/dev/null
            ;;
    esac
}

echo "Generating notification sounds..."
generate_tone "notifications/bell.mp3" 800 1.5 0.05 0.3
generate_tone "notifications/chime.mp3" 1200 1.2 0.1 0.4
generate_tone "notifications/ding.mp3" 1600 0.8 0.02 0.2
generate_tone "notifications/gentle-bell.mp3" 600 2.0 0.2 0.5
generate_tone "notifications/wooden-block.mp3" 400 0.3 0.01 0.05

echo "Generating background sounds..."
generate_noise "background/white-noise.mp3" white 60
generate_noise "background/brown-noise.mp3" brown 60

generate_tone "background/rain.mp3" 100 60 1 1
generate_tone "background/forest.mp3" 200 60 1 1
generate_tone "background/ocean.mp3" 50 60 2 2
generate_tone "background/cafe.mp3" 300 60 1 1
generate_tone "background/fireplace.mp3" 150 60 1 1
generate_tone "background/thunderstorm.mp3" 80 60 1 1

echo "Test sounds generated successfully!"
echo "Note: These are basic synthetic sounds for testing."
echo "For production, replace with high-quality audio files."