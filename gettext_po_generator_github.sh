#!/bin/bash

# Captura a língua original ou define 'en-US' como padrão
OriginalLang=${OriginalLang:-'en'}

DIR="$1"
DIRNAME="$1"
LANGUAGES="bg cs da de el en es et fi fr he hr hu is it ja ko nl no pl pt ro ru sk sv tr uk zh"
# LANGUAGES="pt de es fr"

if [ -z "$OPENAI_KEY" ];then
    red='\e[31;1m'
    neg='\e[37;1m'
    std='\e[m'
    echo -e "${red}Atualize o workflow de tradução.\nO exemplo se encontra em \"https://github.com/biglinux/biglinux-package-with-translate/blob/main/.github/workflows/translate-and-build-package.yml\" ${std}"
    echo
    echo -e "${red}Atualize o workflow de tradução.\nO exemplo se encontra em \"https://github.com/biglinux/biglinux-package-with-translate/blob/main/.github/workflows/translate-and-build-package.yml\" ${std}"
    echo -e "${red}Atualize o workflow de tradução.\nO exemplo se encontra em \"https://github.com/biglinux/biglinux-package-with-translate/blob/main/.github/workflows/translate-and-build-package.yml\" ${std}"
    echo
    echo -e "${red}Atualize o workflow de tradução.\nO exemplo se encontra em \"https://github.com/biglinux/biglinux-package-with-translate/blob/main/.github/workflows/translate-and-build-package.yml\" ${std}"
    sleep infinity
fi

# Detect if folder use subfolder
[ -d "$DIR/$DIR" ] && DIR="$DIR/$DIR"

# Folder locale
[ ! -d $DIR/locale ] && mkdir -p $DIR/locale

# BKP to compare with diff
# cp -R $DIR/locale $DIR/locale-old

# Remove old pot
[ -e $DIR/locale/$DIRNAME.pot ] && rm $DIR/locale/$DIRNAME.pot
echo -e "Directory:\t$DIR"

#######################
# Translate shellscript
#######################
for f in $(find $DIR \( -path "*/.git" -o -path "*/.github" \) -prune -o -type f);do

    # Search shell script
    [ "$(file -b --mime-type $f)" != "text/x-shellscript" ] && continue
    [ $(grep 'git' <<< $f) ] && continue

    # Create .pot file
    echo -e "File:\t\t$f"
    bash --dump-po-strings $f >> $DIR/locale/$DIRNAME-tmp.pot
    [ "$?" != "0" ] && exit 1
done

# Fix pot file
xgettext --package-name="$DIRNAME" --no-location -L PO -o "$DIR/locale/$DIRNAME.pot" -i "$DIR/locale/$DIRNAME-tmp.pot"
rm $DIR/locale/$DIRNAME-tmp.pot


npm install -g stonejs-tools
wget https://raw.githubusercontent.com/biglinux/stonejs-tools/master/src/extract.js -O /usr/local/lib/node_modules/stonejs-tools/src/extract.js

# Search HTML and JS
HTML_JS_FILES=$(find $DIR -type f \( -iname "*.html" -o -iname "*.js" \))

if [ -n "$HTML_JS_FILES" ]; then
    ADD_JSON="json" # Enable to create .json translations for use on html/js
    stonejs extract $HTML_JS_FILES $DIR/locale/$DIRNAME-tmp.pot

    xgettext --package-name="$DIRNAME" --no-location -L PO -o "$DIR/locale/$DIRNAME-js.pot" -i "$DIR/locale/$DIRNAME-tmp.pot"
    rm $DIR/locale/$DIRNAME-tmp.pot

    # Combine files from bash and js/html
    if [[ -e "$DIR/locale/$DIRNAME-js.pot" ]]; then
        if [[ -e "$DIR/locale/$DIRNAME.pot" ]]; then
            mv "$DIR/locale/$DIRNAME.pot" "$DIR/locale/$DIRNAME-bash.pot"
            msgcat --no-wrap --strict "$DIR/locale/$DIRNAME-bash.pot" -i "$DIR/locale/$DIRNAME-js.pot" > $DIR/locale/$DIRNAME-tmp.pot
            xgettext --package-name="$DIRNAME" --no-location -L PO -o "$DIR/locale/$DIRNAME.pot" -i "$DIR/locale/$DIRNAME-tmp.pot"
            rm "$DIR/locale/$DIRNAME-bash.pot"
            rm "$DIR/locale/$DIRNAME-js.pot"
        else
            mv "$DIR/locale/$DIRNAME-js.pot" "$DIR/locale/$DIRNAME.pot"
        fi
    fi
fi


###############
# Translate QML
###############
QML_FILES=$(find $DIR -type f \( -iname "*.qml" \))

if [ -n "$QML_FILES" ]; then

    echo "$QML_FILES" | while read -r file; do
        # Get relative path
        rel_path=$(realpath --relative-to="$DIR" "$file")
        echo "Processing: $rel_path"

        # Extract strings from i18n, i18nc, and qsTr
        awk -v file="$rel_path" '
        BEGIN {
            in_string = 0
            multiline_string = ""
            start_line = 0
        }
        
        function process_string(str, line_no) {
            gsub(/^["'\''"]|["'\''"]$/, "", str)  # Remove outer quotes
            gsub(/\\["'\'']/, "\"", str)          # Escape quotes for PO file
            
            if (str != "") {
                print "#: " file ":" line_no
                
                # Split into lines and format each line
                n = split(str, lines, /\n/)
                if (n == 1) {
                    print "msgid \"" str "\""
                } else {
                    print "msgid \"" lines[1] "\\n\""
                    for (i = 2; i <= n; i++) {
                        if (i == n) {
                            print "\"" lines[i] "\""
                        } else {
                            print "\"" lines[i] "\\n\""
                        }
                    }
                }
                print "msgstr \"\"\n"
            }
        }
        
        {
            line = $0
            line_number = NR
            
            if (!in_string) {
                # Procura por início de i18nc
                if (match(line, /i18nc[ ]*\([^,]*,[ ]*["'\'']/, arr)) {
                    in_string = 1
                    start_line = NR
                    start_pos = RSTART + RLENGTH - 1
                    multiline_string = substr(line, start_pos + 1)
                }
                # Procura por início de i18n/qsTr
                else if (match(line, /(i18n|qsTr)[ ]*\(["'\'']/, arr)) {
                    in_string = 1
                    start_line = NR
                    start_pos = RSTART + RLENGTH - 1
                    multiline_string = substr(line, start_pos + 1)
                }
            } else {
                multiline_string = multiline_string "\n" line
            }
            
            if (in_string) {
                # Procura pelo fechamento da string
                if (match(multiline_string, /([^\\]|^)["'\'']/, arr)) {
                    in_string = 0
                    end_pos = RSTART + RLENGTH - 1
                    complete_string = substr(multiline_string, 1, end_pos - 1)
                    process_string(complete_string, start_line)
                    multiline_string = ""
                }
            }
        }' "$file" >> $DIR/locale/$DIRNAME-tmp.pot
    done

    # Method 3 Fix pot file
    xgettext --package-name="$DIRNAME" --no-location -L PO -o "$DIR/locale/$DIRNAME-qml.pot" -i "$DIR/locale/$DIRNAME-tmp.pot"
    rm $DIR/locale/$DIRNAME-tmp.pot

    # Combine files from bash and js/html
    if [[ -e "$DIR/locale/$DIRNAME-qml.pot" ]]; then
        if [[ -e "$DIR/locale/$DIRNAME.pot" ]]; then
            mv "$DIR/locale/$DIRNAME.pot" "$DIR/locale/$DIRNAME-bash.pot"
            msgcat --no-wrap --strict "$DIR/locale/$DIRNAME-bash.pot" -i "$DIR/locale/$DIRNAME-qml.pot" > $DIR/locale/$DIRNAME-tmp.pot
            xgettext --package-name="$DIRNAME" --no-location -L PO -o "$DIR/locale/$DIRNAME.pot" -i "$DIR/locale/$DIRNAME-tmp.pot"
            rm "$DIR/locale/$DIRNAME-bash.pot"
            rm "$DIR/locale/$DIRNAME-qml.pot"
        else
            mv "$DIR/locale/$DIRNAME-qml.pot" "$DIR/locale/$DIRNAME.pot"
        fi
    fi

fi

###############
# Translate .py
###############
# Install .py dependencies
# sudo pip install python-gettext
# Search strings to translate
for f in $(find $DIR -type f \( -iname "*.py" \));do

    # Search python script
    # [ "$(file -b --mime-type $f)" != "text/x-script.python" ] && continue
    # [ $(grep 'git' <<< $f) ] && continue

    [ ! -e "$DIR/locale/$DIRNAME.pot" ] && >"$DIR/locale/$DIRNAME.pot"
    # Create .pot file
    echo -e "File:\t\t$f"
    xgettext -o "$DIR/locale/python.pot" $f
    #pygettext3 -o "$DIR/locale/python.pot" $f
    #/usr/lib/python3.10/Tools/i18n/pygettext.py -o "$DIR/locale/python.pot" $f
    msgcat --no-wrap --strict "$DIR/locale/$DIRNAME.pot" -i "$DIR/locale/python.pot" > $DIR/locale/$DIRNAME-tmp.pot
    xgettext --package-name="$DIRNAME" --no-location -L PO -o "$DIR/locale/$DIRNAME.pot" -i "$DIR/locale/$DIRNAME-tmp.pot"
    rm $DIR/locale/$DIRNAME-tmp.pot
#     [ "$?" != "0" ] && exit 1
    rm -f "$DIR/locale/python.pot"
done

# Make original lang based in .pot
msgen "$DIR/locale/$DIRNAME.pot" > "$DIR/locale/$OriginalLang.po"

# Remove date
sed -i '/"POT-Creation-Date:/d;/"PO-Revision-Date:/d' $DIR/locale/*

# # Add Subscription-Region support and use brazilsouth
# if [ "$(grep 'Ocp-Apim-Subscription-Region' /usr/local/lib/node_modules/attranslate/dist/services/azure-translator.js)" = "" ]; then
#     sudo sed -i '/Ocp-Apim-Subscription-Key/a "Ocp-Apim-Subscription-Region": "brazilsouth",' /usr/local/lib/node_modules/attranslate/dist/services/azure-translator.js
# fi

# sudo sed -i '/temperature:/s/temperature:.*/temperature: 0,/' /usr/local/lib/node_modules/attranslate/dist/services/openai-translate.js
# sudo sed -i 's/Translate the following text from ${args.srcLng} to ${args.targetLng}/please dont interact, just translate this word or phrase, if you only have one word, just answer me the translation of that word, dont write the original word, translate from ${args.srcLng} to ${args.targetLng}:/' /usr/local/lib/node_modules/attranslate/dist/services/openai-translate.js
# sudo sed -i 's/gpt-3.5-turbo-instruct/gpt-3.5-turbo-0125/' /usr/local/lib/node_modules/attranslate/dist/services/openai-translate.js
# sudo sed -i 's/openai.createCompletion/openai.createChatCompletion/' /usr/local/lib/node_modules/attranslate/dist/services/openai-translate.js
# sudo sed -i 's/completion.data.choices[0].text/chatCompletion.data.choices[0].message/' /usr/local/lib/node_modules/attranslate/dist/services/openai-translate.js

# sudo sed -i 's/Translate the following text from ${args.srcLng} into ${args.targetLng}:/only translate my software string from ${args.srcLng} to ${args.targetLng}. dont chat or explain. Using the correct terms for computer software in the target language, only show target language never repeat string. if you dont find something to translate, dont respond, string:/' /usr/local/lib/node_modules/attranslate/dist/services/openai-translate.js


# cat /usr/local/lib/node_modules/attranslate/dist/services/openai-translate.js

for i in $LANGUAGES; do
    if [ "$i" != "$OriginalLang" ]; then
        attranslate --srcFile=$DIR/locale/$OriginalLang.po --srcLng=$OriginalLang --srcFormat=po --targetFormat=po --service=openai --serviceConfig=$OPENAI_KEY --targetFile=$DIR/locale/$i.po --targetLng=$i
    fi

    sed -i '/Content-Type: text\/plain;/s/charset=.*\\/charset=utf-8\\/' $DIR/locale/$i.po

    # Make .mo
    LANGUAGE_UNDERLINE="$(echo $i | sed 's|-|_|g')"
    mkdir -p $DIR/usr/share/locale/$LANGUAGE_UNDERLINE/LC_MESSAGES
    # Make json translations ( Only if $2 is json word )
    if [[ "$ADD_JSON" == "json" ]]; then
        if [[ -e "$DIR/locale/$i.po" ]]; then
            stonejs build --format=json --merge "$DIR/locale/$i.po" "$DIR/locale/$i.json"
            sed -i "s|^{\"$i\"|{\"$DIR\"|g;s|^{\"C\"|{\"$i\"|g" "$DIR/locale/$i.json"
        else
            rm -f "$DIR/locale/$i.json"
        fi
        cp "$DIR/locale/$i.json" "$DIR/usr/share/locale/$LANGUAGE_UNDERLINE/LC_MESSAGES/$DIRNAME.json"
    fi

    msgfmt "$DIR/locale/$i.po" -o "$DIR/usr/share/locale/$LANGUAGE_UNDERLINE/LC_MESSAGES/$DIRNAME.mo" || true
#     [ "$?" != "0" ] && exit 1
    echo "/usr/share/locale/$LANGUAGE_UNDERLINE/LC_MESSAGES/$DIRNAME.mo"

#     sleep 2
done


