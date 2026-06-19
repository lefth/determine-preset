# Determine x265 encoding preset

Do you want to know how video files were encoded?

x265 presets like "medium" and "veryfast" imply sets of encoding parameters
like lookahead-slices, bframes, and limit-refs. The preset is not stored in the
output file, but the encoding parameters are. By comparing the parameters to
the table in the [x265 documentation](https://x265.readthedocs.io/en/master/presets.html), the preset can be determined.

After installing mediainfo, get the preset by running: `determine-preset video.mp4`
