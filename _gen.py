import io, zlib, base64
css_path = r'C:\laragon\www\praisechords-rs\static\chordpro.css'
js_path = r'C:\laragon\www\praisechords-rs\static\chord-parser.js'
css = open(css_path, encoding='utf-8').read()
js = open(js_path, encoding='utf-8').read()
css_b64 = base64.b64encode(css.encode('utf-8')).decode()
js_b64 = base64.b64encode(js.encode('utf-8')).decode()
html = '''<!DOCTYPE html>
<html><head><meta charset="utf-8">
<style>
body{font-family:sans-serif;margin:10px;}
.col{width:600px;border:1px solid #ccc;padding:8px;margin:6px;display:block;float:left;clear:none}
.rowbox .keep-together{display:inline-flex;flex-direction:row;white-space:nowrap;vertical-align:bottom;}
.colbox  .keep-together{display:inline-flex;flex-direction:column;white-space:nowrap;}
pre{white-space:pre-wrap;background:#eee;font-size:10px;clear:both}
</style></head><body>
<h3>ROW (fix):</h3><div class="col rowbox" id="r1"></div>
<h3>COLUMN (old):</h3><div class="col colbox" id="r2"></div>
<div id="m" style="clear:both"></div>
<script>
var CSSB64="__CSS__";
var JSB64="__JS__";
function base64ToStr(s){var bin=atob(s);var out=[];for(var i=0;i<bin.length;i++)out.push(String.fromCharCode(bin.charCodeAt(i)));return out.join('');}
var styleEl=document.createElement('style');styleEl.textContent=base64ToStr(CSSB64)+'\n.rowbox .keep-together{display:inline-flex;flex-direction:row;white-space:nowrap;vertical-align:bottom;}\n.colbox .keep-together{display:inline-flex;flex-direction:column;white-space:nowrap;}';document.head.appendChild(styleEl);
eval(base64ToStr(JSB64));
var CP="[F]    Quem \u00e9 co[Bb]mo Tu ?\r\nE Q[F]uem daria a [Bb]sua vida por [Dm]mim ?\r\n[F]     Como re[Bb]tribuir, [Dm]   Toda a cria[Bb]\u00e7\u00e3o espera em [Dm]Ti\r\nTudo sus[Gm]tentas com Tua [Csus]m\u00e3o";
document.getElementById('r1').innerHTML=window.parseChordPro2(CP,0,0);
document.getElementById('r2').innerHTML=window.parseChordPro2(CP,0,0);
function meas(root){return Array.prototype.map.call(root.querySelectorAll('.chord-block .lyric'),function(b){var r=b.getBoundingClientRect();return (b.textContent.trim()||'(esp)')+':'+Math.round(r.top)+','+Math.round(r.left);}).join(' | ');}
document.getElementById('m').innerHTML='<pre>ROW: '+meas(document.getElementById('r1'))+'<br>COLUMN: '+meas(document.getElementById('r2'))+'</pre>';
</script>
</body></html>
'''.replace('__CSS__', css_b64).replace('__JS__', js_b64)
open(r'C:\laragon\www\praisechords-rs\layout_test.html','w',encoding='utf-8').write(html)
print('written')
