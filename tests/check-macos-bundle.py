"""Verify a packaged plugin and load/register its native architecture without AE."""
from pathlib import Path
import ctypes as c
import platform,plistlib,subprocess,sys
bundle=Path(sys.argv[1]).resolve()
with (bundle/'Contents/Info.plist').open('rb') as f: info=plistlib.load(f)
assert info['CFBundleIdentifier']=='com.gavinbarnett.chromaticae'
assert info['CFBundleVersion']=='2.2.0'
assert info['CFBundleExecutable']=='Chromatic AE'
binary=bundle/'Contents/MacOS'/info['CFBundleExecutable']
archs=subprocess.check_output(['lipo','-archs',str(binary)],text=True).split()
assert set(archs)=={'arm64','x86_64'},archs
subprocess.run(['codesign','--verify','--deep','--strict','--verbose=2',str(bundle)],check=True)
resource=(bundle/'Contents/Resources/Chromatic AE.rsrc').read_bytes()
for marker in (b'mi64',b'ma64',b'EffectMain',b'ChromaticAE RGB Split'): assert marker in resource,marker
library=c.CDLL(str(binary))
getattr(library,'EffectMain')
callback=c.CFUNCTYPE(c.c_int32,c.c_void_p,c.c_char_p,c.c_char_p,c.c_char_p,c.c_char_p,c.c_int32,c.c_int32,c.c_int32,c.c_int32,c.c_char_p)
seen=[]
@callback
def register(ctx,name,match,category,entry,kind,major,minor,reserved,url):
    seen.append((name,match,category,entry,major,minor,reserved));return 0
entry=library.PluginDataEntryFunction2
entry.argtypes=[c.c_void_p,callback,c.c_void_p,c.c_char_p,c.c_char_p];entry.restype=c.c_int32
assert entry(None,register,None,b'After Effects',b'26.0')==0
assert seen==[(b'Chromatic AE',b'ChromaticAE RGB Split',b'Chromatic',b'EffectMain',13,28,8)],seen
print(f'PASS universal bundle, signature, PiPL and native registration on {platform.machine()}')
subprocess.run(['otool','-L',str(binary)],check=True)
