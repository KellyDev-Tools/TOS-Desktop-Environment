import os
import re
import glob

def process_file(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    orig_content = content

    # 1. Add aria-label based on title for buttons
    # Regex to find <button ... title="X" ...> where aria-label is missing
    def replace_title_btn(match):
        full_tag = match.group(0)
        if 'aria-label=' not in full_tag:
            # extract title
            title_match = re.search(r'title="([^"]+)"', full_tag)
            if title_match:
                title = title_match.group(1)
                # insert aria-label after title
                full_tag = full_tag.replace(f'title="{title}"', f'title="{title}" aria-label="{title}"')
        return full_tag

    content = re.sub(r'<button\s+[^>]*title="[^"]*"[^>]*>', replace_title_btn, content)

    # 2. Add role="button" tabindex="0" to div/span with onclick
    def replace_onclick_div(match):
        full_tag = match.group(0)
        # Check if it has onclick but no role
        if 'onclick=' in full_tag and 'role=' not in full_tag:
            # insert role="button" tabindex="0" before onclick
            full_tag = full_tag.replace('onclick=', 'role="button" tabindex="0" onclick=')
        return full_tag

    content = re.sub(r'<(div|span)\s+[^>]*onclick=[^>]*>', replace_onclick_div, content)

    # 3. Add aria-roledescription="chip" to things with class="...chip..."
    def replace_chip(match):
        full_tag = match.group(0)
        if 'aria-roledescription="chip"' not in full_tag:
            full_tag = full_tag.replace('class="', 'aria-roledescription="chip" class="')
        return full_tag

    content = re.sub(r'<(div|button|span)\s+[^>]*class="[^"]*chip[^"]*"[^>]*>', replace_chip, content)

    if content != orig_content:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"Updated {filepath}")

if __name__ == "__main__":
    src_dir = "/home/tim/TOS-Desktop-Environment/face-svelte-ui/src"
    for root, _, files in os.walk(src_dir):
        for f in files:
            if f.endswith('.svelte'):
                process_file(os.path.join(root, f))
