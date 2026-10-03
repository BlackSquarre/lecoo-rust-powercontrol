# C 版功率与布局调整

使用内置 GPT Image 编辑 revision-3/c-narrow.png。设计效果图，数值为示例，不是运行界面截图。

设置齿轮移到页脚型号右侧，删除顶部独立设置行。四格按温度、CPU Package 功率、风扇转速、内存与磁盘组合排列。保留收窄宽度与无端点文字的滑条。

```text
Use case: ui-mockup / precise-object-edit. Edit this exact reference image, the narrowed C layout of "Lecoo Rust PowerControl", into ONE updated light-theme Windows native desktop utility mockup. Preserve its compact narrow window width, its restrained light palette, Segoe UI / Microsoft YaHei UI type, subtle rounded borders, power/fan radio controls, and overall hierarchy. Flat screenshot, entire window visible with a small neutral outer margin; no presentation annotations or perspective.

ONLY requested layout/content changes:
1. REMOVE the gear button and the ENTIRE EMPTY HEADER STRIP currently between the titlebar and the four status tiles. Move the status grid up just below the titlebar with normal 12–16 logical pixel top padding. Window becomes shorter accordingly.
2. MOVE that SINGLE settings gear to the BOTTOM footer at the FAR RIGHT, to the RIGHT of the device model text "Lecoo MINI PRO-AHP", aligned on the SAME ROW with that model. It must NOT occupy a separate row or large new footer section. Small discreet gear-only native button around 28–32 logical pixels, not a large square tile. Device footer keeps model first line and "AMD Ryzen 7 8745H" second line on the LEFT; button baseline/center aligns with model first line. Exactly ONE gear in the entire image.
3. Preserve the 2x2 status grid and change tile contents as follows, precisely in these positions:
TOP LEFT: heading "CPU 温度", large "41.0 °C".
TOP RIGHT: replace former RPM tile with heading "CPU Package 功率", large "18.8 W". Heading should be smaller so it fits a SINGLE line at this narrow width. 18.8 W is illustrative, not a live measurement.
BOTTOM LEFT: replace former memory tile with heading "风扇转速", large "1541 RPM".
BOTTOM RIGHT: combine MEMORY AND DISK in this SINGLE FOURTH TILE. It contains two neat vertically stacked compact subrows, each with its own label, readable value and thin blue native progress bar: first "内存" with "81% · 7.1/8.8 GiB" and 81% progress; second "磁盘" with "40% · 192/476 GiB" and 40% progress. Both complete readings and both progress bars MUST be visible in that one tile. Use smaller type and tighter spacing here, without clipping or requiring a larger window width. It must not become two separate fifth/sixth tiles. All four outer tiles have equal width and height, matched between rows, within original narrow window.
4. Everything below the grid remains as reference: "电源模式" with single-row radios "安静" GREEN, "均衡" BLUE, "性能" RED selected, balanced left/right inner padding ending after "性能"; no redundant current-mode text. Then "风扇控制" with single-row radios "自动" "手动" "最大", manual selected blue. Below that, blue horizontal native trackbar and "50%" at its right on same row. KEEP THE ENTIRE ENDPOINT LABEL ROW DELETED: no "35%", no "100%", no other captions below slider. The slider's thumb represents value 50 on underlying range 35–100, around 23% travel. No Apply button.
Do not add a standalone "设置" heading, extra settings panel, status/error copy, refresh/reconnect/exit/tray buttons, explanatory captions, version footer, glow/blur/acrylic/3D or giant empty spaces. Do not put gear in titlebar or upper right. Crisp legible Chinese and practical native control sizes. Make footer compact and main window visibly shorter by eliminating old settings strip.
```
