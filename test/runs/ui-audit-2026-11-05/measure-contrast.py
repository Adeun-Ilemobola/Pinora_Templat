"""Reproduce opaque token-pair contrasts; no browser or product modification."""
import json
import math
from pathlib import Path

def linear_rgb(lch):
    light, chroma, hue = lch
    a = chroma * math.cos(math.radians(hue))
    b = chroma * math.sin(math.radians(hue))
    l = (light + 0.3963377774*a + 0.2158037573*b)**3
    m = (light - 0.1055613458*a - 0.0638541728*b)**3
    s = (light - 0.0894841775*a - 1.2914855480*b)**3
    return [max(0,min(1,v)) for v in (
        4.0767416621*l - 3.3077115913*m + 0.2309699292*s,
        -1.2684380046*l + 2.6097574011*m - 0.3413193965*s,
        -0.0041960863*l - 0.7034186147*m + 1.7076147010*s)]

def luminance(lch):
    return sum(x*y for x,y in zip(linear_rgb(lch),[0.2126,0.7152,0.0722]))

pairs = {
    'light muted text / card': ((.58,.031,107.3),(1,0,0)),
    'light muted text / muted surface': ((.58,.031,107.3),(.966,.005,106.5)),
    'light primary text / primary': ((.98,.016,73.684),(.553,.195,38.402)),
    'dark muted text / card': ((.737,.021,106.9),(.228,.013,107.4)),
    'dark primary text / primary': ((.98,.016,73.684),(.47,.157,37.304)),
}
results = []
for name,(foreground,background) in pairs.items():
    lo,hi = sorted([luminance(foreground),luminance(background)])
    ratio=(hi+.05)/(lo+.05)
    results.append({'pair':name,'ratio':round(ratio,3),'normal_text_4_5':ratio>=4.5})
output=json.dumps(results,indent=2)
Path(__file__).with_name('contrast-results.json').write_text(output+'\n',encoding='utf-8')
print(output)
