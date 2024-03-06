import subprocess
import matplotlib.pyplot as plt

SEG_LENS = [1 << i for i in range(6, 16)]

prove = []
verify = []
sizes = []
for seg_len in SEG_LENS:
    print(f"Running for seg_len = {seg_len}")
    res = subprocess.run(["cargo", "bench", "--bench", "fib_prove_segmented", "--", "--segment-len", str(seg_len)], capture_output=True).stdout.decode("utf-8")
    prove.append(int(res.split('=')[1]) / 1024)
    
    res = ''.join(subprocess.run(["cargo", "bench", "--bench", "fib_verify_segmented", "--", "--segment-len", str(seg_len)], capture_output=True).stdout.decode("utf-8"))
    size, mem = res.split()
    sizes.append(int(size.split('=')[1])/ 1024)
    verify.append(int(mem.split('=')[1]) / 1024)

print(f"segments={SEG_LENS}")
print(f"prove={prove}")
print(f"verify={verify}")
print(f"size={sizes}")


