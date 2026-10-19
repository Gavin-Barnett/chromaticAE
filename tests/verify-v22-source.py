"""Protect the released Windows behaviour while adding Mac packaging."""
from pathlib import Path
import hashlib
root=Path(__file__).resolve().parents[1]
expected={'native/src/lib.rs': '9579d074c6e47cc6caa1cb5abfa7baf43dd2956344e1c79fb3861cc684f1fadb', 'native/src/engine.rs': '651983b289ade25ae039ea4db2941bf0c2a8d605adcc67b906ff781899e5ecf8', 'native/Cargo.toml': 'd48693dda9bcfdbb6883ab3f9cc8d9663c0330944cdd294868cce1842986e469', 'native/Cargo.lock': '10aca775bb77eb8c35475a04b11d0e891e3e758fe851abc2276acd3c374e86e2', 'plugin/Chromatic AE.aex': 'ae8aac580cc89a8453320e88357389cbdcf978ed984f0c81e4c1fbbeb6c382b7'}
for name,digest in expected.items():
    data=(root/name).read_bytes()
    if name.endswith(('.rs','.toml','.lock')): data=data.replace(b'\r\n',b'\n')
    actual=hashlib.sha256(data).hexdigest()
    assert actual==digest, f'{name} changed from the released v2.2 Windows source/binary'
    print(f'PASS unchanged v2.2: {name}')
