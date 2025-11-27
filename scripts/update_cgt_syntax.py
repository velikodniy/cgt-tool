import re
import os

def update_cgt_syntax(file_path):
    with open(file_path, 'r') as f:
        content = f.read()

    # Regex patterns and replacements
    # DIVIDEND: YYYY-MM-DD DIVIDEND TICKER AMOUNT TAX_PAID -> YYYY-MM-DD DIVIDEND TICKER AMOUNT TAX TAX_PAID
    content = re.sub(r'^(\d{4}-\d{2}-\d{2}\s+DIVIDEND\s+\S+\s+\d+\.?\d*)\s+(\d+\.?\d*)\s*$', r'\1 TAX \2', content, flags=re.MULTILINE)

    # CAPRETURN: YYYY-MM-DD CAPRETURN TICKER AMOUNT EXPENSES -> YYYY-MM-DD CAPRETURN TICKER AMOUNT EXPENSES EXPENSES
    content = re.sub(r'^(\d{4}-\d{2}-\d{2}\s+CAPRETURN\s+\S+\s+\d+\.?\d*)\s+(\d+\.?\d*)\s*$', r'\1 EXPENSES \2', content, flags=re.MULTILINE)

    # SPLIT/UNSPLIT: YYYY-MM-DD (SPLIT|UNSPLIT) TICKER RATIO -> YYYY-MM-DD (SPLIT|UNSPLIT) TICKER RATIO RATIO
    content = re.sub(r'^(\d{4}-\d{2}-\d{2}\s+(SPLIT|UNSPLIT)\s+\S+)\s+(\d+\.?\d*)\s*$', r'\1 RATIO \3', content, flags=re.MULTILINE)

    with open(file_path, 'w') as f:
        f.write(content)

def main():
    script_dir = os.path.dirname(__file__)
    root_dir = os.path.abspath(os.path.join(script_dir, os.pardir))
    tests_data_dir = os.path.join(root_dir, 'tests', 'data')

    for filename in os.listdir(tests_data_dir):
        if filename.endswith('.cgt'):
            file_path = os.path.join(tests_data_dir, filename)
            print(f"Updating {file_path}...")
            update_cgt_syntax(file_path)
    print("All .cgt files updated.")

if __name__ == '__main__':
    main()
