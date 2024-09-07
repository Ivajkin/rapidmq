import subprocess

def run_tests():
    result = subprocess.run("cargo test", shell=True)
    return result.returncode == 0

if run_tests():
    print("Tests passed")
else:
    print("Tests failed")
    exit(1)