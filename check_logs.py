import sys
import linecache

USER_LINE_OFFSET = 0
REF_LINE_OFFSET = 7


def parse_user_line(user_file: str, converted_line_index: int) -> tuple[dict[str, str], str, int]:
    line = linecache.getline(user_file, converted_line_index)
    line = line.strip()
    parts = line.split()
    regs = {}
    for part in parts:
        for prefix in ['A:', 'F:', 'B:', 'C:', 'D:', 'E:', 'H:', 'L:', 'PC:', 'PPU:', 'TOTAL_CY_DOTS:', 'SCANLINE:']:
            if part.startswith(prefix):
                key, val = part.split(':', 1)
                regs[key] = val
                break
    # Check if all required registers are present
    required = ['A', 'F', 'B', 'C', 'D', 'E', 'H', 'L', 'PC', 'PPU', 'TOTAL_CY_DOTS', 'SCANLINE']
    missing = [r for r in required if r not in regs]
    if missing:
        print(f"User log line {converted_line_index} is missing registers: {', '.join(missing)}")
        sys.exit(1)
    return regs, line, converted_line_index


def parse_ref_line(ref_file: str, converted_line_index: int) -> tuple[dict[str, str], str, int]:
    line = linecache.getline(ref_file, converted_line_index)
    stripped = line.strip()
    if stripped.startswith('A:'):
        parts = stripped.split()
        regs = {}
        for part in parts:
            if part.startswith('A:'):
                regs['A'] = part.split(':', 1)[1]
            elif part.endswith(')'):
                regs['TOTAL_CY_DOTS'] = part[:-1]
            elif part.startswith('ppu:'):
                ppu = part.split(':', 1)[1]
                regs['PPU'] = ppu[1:]
            elif part.startswith('F:'):
                f_str = part.split(':', 1)[1]
                if len(f_str) != 4:
                    print(
                        f"Error parsing F flags in reference log line {converted_line_index}: '{f_str}' (expected 4 characters)")
                    sys.exit(1)
                f_val = 0
                if f_str[0] != '-':
                    f_val |= 0x80
                if f_str[1] != '-':
                    f_val |= 0x40
                if f_str[2] != '-':
                    f_val |= 0x20
                if f_str[3] != '-':
                    f_val |= 0x10
                regs['F'] = f"{f_val:02X}"
            elif part.startswith('BC:'):
                bc = part.split(':', 1)[1]
                if len(bc) != 4:
                    print(
                        f"Error parsing BC in reference log line {converted_line_index}: '{bc}' (expected 4 hex digits)")
                    sys.exit(1)
                regs['B'] = bc[:2]
                regs['C'] = bc[2:]
            elif part.startswith('DE:'):
                de = part.split(':', 1)[1]
                if len(de) != 4:
                    print(f"Error parsing DE in reference log line {converted_line_index}: '{de}'")
                    sys.exit(1)
                regs['D'] = de[:2]
                regs['E'] = de[2:]
            elif part.startswith('HL:'):
                hl = part.split(':', 1)[1]
                if len(hl) != 4:
                    print(f"Error parsing HL in reference log line {converted_line_index}: '{hl}'")
                    sys.exit(1)
                regs['H'] = hl[:2]
                regs['L'] = hl[2:]
            elif part.startswith('PC:'):
                pc_part = part.split(':', 1)[1]
                pc_val = pc_part.split()[0]
                regs['PC'] = pc_val
        # Check required registers
        required = ['A', 'F', 'B', 'C', 'D', 'E', 'H', 'L', 'PC']
        missing = [r for r in required if r not in regs]
        if missing:
            print(f"Reference log line {converted_line_index} is missing registers: {', '.join(missing)}")
            sys.exit(1)
        return regs, line, converted_line_index
    sys.exit(1)


def main():
    if len(sys.argv) < 3:
        print("Usage: python compare_logs.py <user_log_file> <reference_log_file> [ignored_lines]")
        sys.exit(1)
    user_log_file = sys.argv[1]
    reference_log_file = sys.argv[2]
    custom_lines = False
    if len(sys.argv) > 3:
        custom_lines = True
        user_line = int(sys.argv[3])
        reference_line = int(sys.argv[4])

    current_user_line_index = 1 + USER_LINE_OFFSET  # We start indexing lines at 1, like file editors do
    current_ref_line_index = 1 + REF_LINE_OFFSET

    if custom_lines:
        current_user_line_index = user_line
        current_ref_line_index = reference_line

    while True:
        user_regs, user_line, user_lineno = parse_user_line(user_log_file, current_user_line_index)
        ref_regs, ref_line, ref_lineno = parse_ref_line(reference_log_file, current_ref_line_index)

        # Compare each register
        for reg in ['A', 'F', 'B', 'C', 'D', 'E', 'H', 'L']:  # 'PC', 'TOTAL_CY_DOTS']:
            u_val = user_regs[reg].upper()
            r_val = ref_regs[reg].upper()
            if u_val != r_val:
                print(
                    f"Mismatch in register {reg} at line User: {current_user_line_index} Ref: {current_ref_line_index}:")
                print(f"User log (line {user_lineno}):\n{user_line}")
                print(f"Reference log (line {ref_lineno}):\n{ref_line}")
                print(f"User value: {u_val}")
                print(f"Reference value: {r_val}")
                sys.exit(1)
        current_user_line_index += 1
        current_ref_line_index += 1


if __name__ == "__main__":
    main()
