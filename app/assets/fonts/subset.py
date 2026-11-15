"""Build small, renamed interface fonts from epaint_default_fonts 0.37 originals."""
from pathlib import Path
import argparse
import shutil
from fontTools import subset
from fontTools.ttLib import TTFont


def build(source, target, name):
    font = TTFont(source, recalcTimestamp=False)
    options = subset.Options()
    options.recalc_timestamp = False
    options.drop_tables += ['TTFA']
    options.name_IDs = ['*']
    options.name_languages = ['*']
    worker = subset.Subsetter(options=options)
    worker.populate(unicodes=list(range(32, 256)) + list(range(0x2000, 0x2070)) + [ord(c) for c in '●■←→↑↓↔↕▶▼◀▲−'])
    worker.subset(font)
    for record in font['name'].names:
        if record.nameID in (1, 2, 3, 4, 6, 16, 17):
            value = 'Regular' if record.nameID in (2, 17) else name
            record.string = value.encode(record.getEncoding())
    font.save(target)
    print(target.name, target.stat().st_size)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path, help='epaint_default_fonts 0.37 fonts directory')
    source = parser.parse_args().source
    output = Path(__file__).parent
    build(source/'Ubuntu-Light.ttf', output/'karaya-ui.ttf', 'KarayaUI')
    build(source/'Hack-Regular.ttf', output/'karaya-mono.ttf', 'KarayaMono')
    for filename in ['egui-icons.ttf', 'egui-icons.txt', 'UFL.txt', 'Hack-Regular.txt']:
        shutil.copyfile(source/filename, output/filename)


if __name__ == '__main__':
    main()
