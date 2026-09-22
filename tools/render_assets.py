import os
import subprocess
from PIL import Image

temp_dir = os.environ.get('TEMP', r'C:\Users\SirMaku\AppData\Local\Temp')
edge_bin = r'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'

def render_svg(svg_content, out_path, w=600, h=400):
    html_path = os.path.join(temp_dir, 'render_tmp.html')
    png_tmp = os.path.join(temp_dir, 'render_tmp.png')
    
    html = f'<!DOCTYPE html><html><body style="margin:0;padding:0;background:transparent;overflow:hidden;">{svg_content}</body></html>'
    with open(html_path, 'w', encoding='utf-8') as f:
        f.write(html)
        
    cmd = [
        edge_bin,
        '--headless=new',
        '--disable-gpu',
        '--no-sandbox',
        '--default-background-color=00000000',
        '--hide-scrollbars',
        f'--window-size={w},{h}',
        f'--screenshot={png_tmp}',
        f'file:///{html_path.replace(os.sep, "/")}'
    ]
    subprocess.run(cmd, check=True, capture_output=True)
    
    im = Image.open(png_tmp)
    bbox = im.getbbox()
    if bbox:
        # Add 2px padding
        x0, y0, x1, y1 = bbox
        x0 = max(0, x0 - 2)
        y0 = max(0, y0 - 2)
        x1 = min(im.width, x1 + 2)
        y1 = min(im.height, y1 + 2)
        cropped = im.crop((x0, y0, x1, y1))
        cropped.save(out_path)
        print(f'Rendered {out_path}: {cropped.size}')
    else:
        print(f'Error: empty bbox for {out_path}')

# 1. Visa: Crisp White version
with open('visa_datatrans.svg', encoding='utf-8') as f:
    visa_svg = f.read()
visa_svg = visa_svg.replace('<rect width="120" height="80" rx="4" fill="white"/>', '')
visa_svg = visa_svg.replace('fill="#1434CB"', 'fill="#FFFFFF"')
visa_svg = visa_svg.replace('width="120" height="80"', 'width="600" height="400"')
render_svg(visa_svg, 'src/assets/visa.png')

# 2. Mastercard
with open('mc_datatrans.svg', encoding='utf-8') as f:
    mc_svg = f.read()
mc_svg = mc_svg.replace('<rect width="120" height="80" rx="4" fill="white"/>', '')
mc_svg = mc_svg.replace('width="120" height="80"', 'width="600" height="400"')
render_svg(mc_svg, 'src/assets/mastercard.png')

# 3. JCB
with open('jcb_datatrans.svg', encoding='utf-8') as f:
    jcb_svg = f.read()
jcb_svg = jcb_svg.replace('<rect width="120" height="80" rx="4" fill="white"/>', '')
jcb_svg = jcb_svg.replace('width="120" height="80"', 'width="600" height="400"')
render_svg(jcb_svg, 'src/assets/jcb.png')

# 4. Contactless Wave Symbol
wave_svg = '''
<svg width="240" height="240" viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
  <path d="M48,15 A42,42 0 0,1 48,85" stroke="white" stroke-width="7" stroke-linecap="round" fill="none"/>
  <path d="M37,27 A30,30 0 0,1 37,73" stroke="white" stroke-width="7" stroke-linecap="round" fill="none"/>
  <path d="M26,38 A18,18 0 0,1 26,62" stroke="white" stroke-width="7" stroke-linecap="round" fill="none"/>
  <path d="M15,48 A6,6 0 0,1 15,52" stroke="white" stroke-width="7" stroke-linecap="round" fill="none"/>
</svg>
'''
render_svg(wave_svg, 'src/assets/contactless.png', 240, 240)

# 4. Authentic Gold EMV Chip
# Create a stunning 400x300 high resolution golden EMV smartcard contact pad with realistic trace lines and specular reflection
chip_svg = '''
<svg width="400" height="300" viewBox="0 0 200 150" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <linearGradient id="gold_grad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#FFF2A8"/>
      <stop offset="25%" stop-color="#E5BE4A"/>
      <stop offset="50%" stop-color="#C69527"/>
      <stop offset="75%" stop-color="#E5BE4A"/>
      <stop offset="100%" stop-color="#9E7012"/>
    </linearGradient>
    <linearGradient id="inner_gold" x1="0%" y1="100%" x2="100%" y2="0%">
      <stop offset="0%" stop-color="#FCEBA0"/>
      <stop offset="50%" stop-color="#DDAF38"/>
      <stop offset="100%" stop-color="#B8851B"/>
    </linearGradient>
    <filter id="chip_drop" x="-10%" y="-10%" width="130%" height="130%">
      <feDropShadow dx="2" dy="3" stdDeviation="3" flood-color="#000000" flood-opacity="0.45"/>
    </filter>
  </defs>
  <!-- Main chip body with rounded corners and drop shadow -->
  <rect x="8" y="8" width="184" height="134" rx="16" fill="url(#gold_grad)" filter="url(#chip_drop)" stroke="#7A560D" stroke-width="1.5"/>
  <!-- Subtly indented contact pattern paths (carved trace lines) -->
  <!-- Center contact rectangle -->
  <rect x="74" y="44" width="52" height="62" rx="10" fill="url(#inner_gold)" stroke="#684705" stroke-width="1.8"/>
  <line x1="100" y1="44" x2="100" y2="106" stroke="#684705" stroke-width="1.8"/>
  <!-- Horizontal partition lines -->
  <line x1="8" y1="52" x2="74" y2="52" stroke="#684705" stroke-width="1.8"/>
  <line x1="126" y1="52" x2="192" y2="52" stroke="#684705" stroke-width="1.8"/>
  <line x1="8" y1="98" x2="74" y2="98" stroke="#684705" stroke-width="1.8"/>
  <line x1="126" y1="98" x2="192" y2="98" stroke="#684705" stroke-width="1.8"/>
  <!-- Vertical top/bottom partition lines -->
  <line x1="100" y1="8" x2="100" y2="44" stroke="#684705" stroke-width="1.8"/>
  <line x1="100" y1="106" x2="100" y2="142" stroke="#684705" stroke-width="1.8"/>
</svg>
'''
render_svg(chip_svg, 'src/assets/emv_chip.png', 400, 300)

print('All assets rendered successfully!')
