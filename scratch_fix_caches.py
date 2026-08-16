import os
import glob

workflows = glob.glob('.github/workflows/*.yml')
if not workflows:
    print("No workflows found!")
    exit(1)

old_cache_block = """      - name: Rust Cache
        uses: Swatinem/rust-cache@v2
        with:
          shared-key: "kestrel-${{ github.job }}"
          save-if: ${{ github.ref == 'refs/heads/dev' || github.ref == 'refs/heads/main' }}"""
          
new_cache_block = """      - name: Rust Cache
        uses: Swatinem/rust-cache@v2
        with:
          shared-key: "kestrel-rust-shared-cache"
          save-if: false"""

for file in workflows:
    # Skip cache-warmer.yml since it's the one we just created
    if 'cache-warmer.yml' in file:
        continue
        
    with open(file, 'r') as f:
        content = f.read()

    # Replace all cache blocks with the standard shared one
    content = content.replace(old_cache_block, new_cache_block)
    
    # Now, for Android jobs, we want to remove the cache block that appears right after checkout,
    # and insert it AFTER `rustup target add`.
    # It's easiest to process line by line.
    lines = content.split('\n')
    new_lines = []
    
    in_android_job = False
    skip_next = 0
    
    for i, line in enumerate(lines):
        if skip_next > 0:
            skip_next -= 1
            continue
            
        if 'name: Build Mail (Android)' in line or 'name: Build Calendar (Android)' in line:
            in_android_job = True
            
        # If we are in an android job, and we hit the rust cache block, skip it
        if in_android_job and 'name: Rust Cache' in line:
            # Skip the next 5 lines
            skip_next = 5
            continue
            
        new_lines.append(line)
        
        # When we hit the target add line, insert the Android specific cache block
        if 'rustup target add aarch64-linux-android' in line:
            new_lines.append('')
            new_lines.append('      - name: Rust Cache')
            new_lines.append('        uses: Swatinem/rust-cache@v2')
            new_lines.append('        with:')
            new_lines.append('          shared-key: "kestrel-rust-android-cache"')
            new_lines.append('          save-if: false')
            in_android_job = False # Reset for the next job
            
    with open(file, 'w') as f:
        f.write('\n'.join(new_lines))

print("Workflows updated successfully")
