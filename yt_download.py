import time

with open("local.txt", "w") as f:
    f.write(f"accessed at {time.time()}")