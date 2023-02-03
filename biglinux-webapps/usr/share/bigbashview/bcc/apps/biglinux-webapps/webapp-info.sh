#!/usr/bin/env bash

#Translation

export TEXTDOMAINDIR="/usr/share/locale"
export TEXTDOMAIN=biglinux-webapps

. ./scripts/detect_browser.env

DESKNAME=${filedesk##*/}
USER_DESKTOP=$(xdg-user-dir DESKTOP)
NAME=$(awk -F'=' '/Name/{print $2}' "$filedesk")
ICON=$(awk -F'=' '/Icon/{print $2}' "$filedesk")
CATEGORY=$(awk -F'=' '/Categories/{print $2}' "$filedesk")
EXEC=$(awk '/Exec/{print $0}' "$filedesk")

if grep -q '.local.bin' <<<"$EXEC"; then
  BIN=$(awk -F'=' '{print $2}' <<<"$EXEC")
  URL=$(awk '/new-instance/{gsub(/"/, "");print $9}' "$BIN")
  BROWSER=$(awk '/new-instance/{print $3}' "$BIN")
else
  URL=$(awk -F'app=' '{print $2}' <<<"$EXEC")
  BROWSER=$(awk '{gsub(/Exec=/, "");print $1}' <<<"$EXEC")
  if [ ! "$URL" ]; then
    URL=$(awk '{print $4}' <<<"$EXEC")
  fi
fi

if grep -q '.var.lib.flatpak.exports.bin' <<<"$BROWSER"; then
  BROWSER=${BROWSER##*/}
fi

if grep -q '..user.data.dir.' <<<"$EXEC"; then
  checked_perfil='checked'
fi

[ -L "$USER_DESKTOP/$DESKNAME" ] && checked='checked'

case "$BROWSER" in
com.brave.Browser) _ICON='brave' ;;
electron*) _ICON='electron' ;;
*)
  _ICON="${BROWSER##*.}"
  _ICON="${_ICON/-browser//}"
  _ICON="${_ICON%-stable}" # removes versioning
  _ICON="${_ICON%-community}"
  _ICON="${_ICON#microsoft-}" # removes branding
  _ICON="${_ICON#google-}"
  _ICON="${_ICON#flashpeak-}"
  _ICON="${_ICON,,}" # to lowercase
  ;;
esac

echo -n '
<div class="content-section">
  <ul style="margin-top:-20px">
    <li>
      <div class="products">
        <svg viewBox="0 0 512 512">
          <path fill="currentColor" d="M512 256C512 397.4 397.4 512 256 512C114.6 512 0 397.4 0 256C0 114.6 114.6 0 256 0C397.4 0 512 114.6 512 256zM57.71 192.1L67.07 209.4C75.36 223.9 88.99 234.6 105.1 239.2L162.1 255.7C180.2 260.6 192 276.3 192 294.2V334.1C192 345.1 198.2 355.1 208 359.1C217.8 364.9 224 374.9 224 385.9V424.9C224 440.5 238.9 451.7 253.9 447.4C270.1 442.8 282.5 429.1 286.6 413.7L289.4 402.5C293.6 385.6 304.6 371.1 319.7 362.4L327.8 357.8C342.8 349.3 352 333.4 352 316.1V307.9C352 295.1 346.9 282.9 337.9 273.9L334.1 270.1C325.1 261.1 312.8 255.1 300.1 255.1H256.1C245.9 255.1 234.9 253.1 225.2 247.6L190.7 227.8C186.4 225.4 183.1 221.4 181.6 216.7C178.4 207.1 182.7 196.7 191.7 192.1L197.7 189.2C204.3 185.9 211.9 185.3 218.1 187.7L242.2 195.4C250.3 198.1 259.3 195 264.1 187.9C268.8 180.8 268.3 171.5 262.9 165L249.3 148.8C239.3 136.8 239.4 119.3 249.6 107.5L265.3 89.12C274.1 78.85 275.5 64.16 268.8 52.42L266.4 48.26C262.1 48.09 259.5 48 256 48C163.1 48 84.4 108.9 57.71 192.1L57.71 192.1zM437.6 154.5L412 164.8C396.3 171.1 388.2 188.5 393.5 204.6L410.4 255.3C413.9 265.7 422.4 273.6 433 276.3L462.2 283.5C463.4 274.5 464 265.3 464 256C464 219.2 454.4 184.6 437.6 154.5H437.6z"/>
        </svg>
        '$"URL:"'
      </div>
      <input type="search" class="input" id="urlDeskEdit" name="urldesk" value="'"$URL"'" readonly/>
      <div class="button-wrapper">
        <div class=app-card>
          <div class="button-wrapper">
            <button class="button" style="height:30px" id="detectAllEdit">'$"Detectar Nome e Ícone"'</button>
          </div>
        </div>
      </div>
    </li>

    <li>
      <div class="products">
        <svg viewBox="0 0 512 512">
          <path fill="currentColor" d="M96 0C113.7 0 128 14.33 128 32V64H480C497.7 64 512 78.33 512 96C512 113.7 497.7 128 480 128H128V480C128 497.7 113.7 512 96 512C78.33 512 64 497.7 64 480V128H32C14.33 128 0 113.7 0 96C0 78.33 14.33 64 32 64H64V32C64 14.33 78.33 0 96 0zM448 160C465.7 160 480 174.3 480 192V352C480 369.7 465.7 384 448 384H192C174.3 384 160 369.7 160 352V192C160 174.3 174.3 160 192 160H448z"/>
        </svg>
        '$"Nome:"'
      </div>
      <input type="search" class="input" id="nameDeskEdit" name="namedesk" value="'"$NAME"'"/>
    </li>

    <li>
      <div class="products">
        <div style="margin-bottom:15px" class="svg-center">
          <div class="iconDetect-display-Edit">
            <div class="iconDetect-remove-Edit">
              <svg viewBox="0 0 448 512" style="width:20px;height:20px;"><path d="M384 32C419.3 32 448 60.65 448 96V416C448 451.3 419.3 480 384 480H64C28.65 480 0 451.3 0 416V96C0 60.65 28.65 32 64 32H384zM143 208.1L190.1 255.1L143 303C133.7 312.4 133.7 327.6 143 336.1C152.4 346.3 167.6 346.3 176.1 336.1L223.1 289.9L271 336.1C280.4 346.3 295.6 346.3 304.1 336.1C314.3 327.6 314.3 312.4 304.1 303L257.9 255.1L304.1 208.1C314.3 199.6 314.3 184.4 304.1 175C295.6 165.7 280.4 165.7 271 175L223.1 222.1L176.1 175C167.6 165.7 152.4 165.7 143 175C133.7 184.4 133.7 199.6 143 208.1V208.1z"/></svg>
            </div>
          </div>
          <img id="iconDeskEdit" src="'"$ICON"'" width="58" height="58" />
          <input type="hidden" name="icondesk" value="'"$ICON"'" id="inputIconDeskEdit" />
        </div>
        '$"Ícone do WebApp"'
      </div>
      <div class="button-wrapper">
        <button class="button" style="height:30px" id="loadIconEdit">'$"Alterar"'</button>
      </div>
    </li>

    <li>
      <div class="products">
        <div class="svg-center" id="thumb">
          <img height="58" width="58" id="browserEdit" src="icons/'"$_ICON"'.svg"/>
        </div>
        '$"Navegador"'
      </div>
      <div class="button-wrapper">
        <select class="svg-center" id="browserSelectEdit" name="browserNew">' | tr -d "\t\n\r"

for browser in "${browser_bin_list[@]}"; do
  binary=$(which "$browser" 2>/dev/null)
  if [ -x "${binary##/usr/local/*}" ] || [ -x "${FLATPAK_BIN}/$browser" ] || [ -x "${SNAPD_BIN}/$browser" ]; then
    echo -n "<option value=\"$browser\" $(
    [ "$BROWSER" = "$browser" ] && echo -n "selected"
  )>${browser_trans["$browser"]}</option>"
  fi
done
# <option '$selected_brave' value="brave">$"BRAVE"</option>
# <option '$selected_chrome' value="google-chrome-stable">$"CHROME"</option>
# <option '$selected_chromium' value="chromium">$"CHROMIUM"</option>
# <option '$selected_edge' value="microsoft-edge-stable">$"EDGE"</option>
# <option '$selected_epiphany' value="epiphany">$"EPIPHANY"</option>
# <option '$selected_firefox' value="firefox">$"FIREFOX"</option>
# <option '$selected_librewolf' value="librewolf">$"LIBREWOLF"</option>
# <option '$selected_vivaldi' value="vivaldi-stable">$"VIVALDI"</option>
# <option '$selected_brave_flatpak' value="com.brave.Browser">$"BRAVE (FLATPAK)"</option>
# <option '$selected_chrome_flatpak' value="com.google.Chrome">$"CHROME (FLATPAK)"</option>
# <option '$selected_chromium_flatpak' value="org.chromium.Chromium">$"CHROMIUM (FLATPAK)"</option>
# <option '$selected_edge_flatpak' value="com.microsoft.Edge">$"EDGE (FLATPAK)"</option>
# <option '$selected_epiphany_flatpak' value="org.gnome.Epiphany">$"EPIPHANY (FLATPAK)"</option>
# <option '$selected_firefox_flatpak' value="org.mozilla.firefox">$"FIREFOX (FLATPAK)"</option>
# <option '$selected_librewolf_flatpak' value="io.gitlab.librewolf-community">$"LIBREWOLF (FLATPAK)"</option>
echo -n '
        </select>
        <input type="hidden" name="browserOld" value="'"$BROWSER"'"/>
        <input type="hidden" name="filedesk" value="'"$filedesk"'"/>
        <input type="hidden" name="categoryOld" value="'"${CATEGORY/;/}"'"/>
        <input type="hidden" name="namedeskOld" value="'"$NAME"'"/>
        <input type="hidden" name="icondeskOld" value="'"$ICON"'"/>
      </div>
    </li>

    <li>
      <div class="products">
        <div class="svg-center" id="imgCategoryEdit">'$(<"./icons/${CATEGORY/;/}.svg")'</div>
        '$"Categoria"'
      </div>
      <div class="button-wrapper">
        <div class="svg-center">
          <select class="svg-center" id="categorySelectEdit" name="category">' | tr -d "\t\n\r"

declare -A categories=(
  ["Development"]=$"DESENVOLVIMENTO"
  ["Office"]=$"ESCRITÓRIO"
  ["Graphics"]=$"GRÁFICOS"
  ["Network"]=INTERNET
  ["Game"]=$"JOGOS"
  ["AudioVideo"]=$"MULTIMÍDIA"
  ["Webapps"]=$"WEBAPPS"
  ["Google"]=$"WEBAPPS GOOGLE"
)

for category in "${!categories[@]}"; do
  echo -n "<option value=\"$category\" $(
    [ "${CATEGORY/;/}" = "$category" ] && echo -n "selected"
  )>${categories["$category"]}</option>"
done

echo -n '
          </select>
        </div>
      </div>
    </li>

    <li>
      <div class="products">
        <svg viewBox="0 0 512 512">
          <path fill="currentColor" d="M464 96h-192l-64-64h-160C21.5 32 0 53.5 0 80v352C0 458.5 21.5 480 48 480h416c26.5 0 48-21.5 48-48v-288C512 117.5 490.5 96 464 96zM336 311.1h-56v56C279.1 381.3 269.3 392 256 392c-13.27 0-23.1-10.74-23.1-23.1V311.1H175.1C162.7 311.1 152 301.3 152 288c0-13.26 10.74-23.1 23.1-23.1h56V207.1C232 194.7 242.7 184 256 184s23.1 10.74 23.1 23.1V264h56C349.3 264 360 274.7 360 288S349.3 311.1 336 311.1z"/>
        </svg>
        '$"Criar atalho na Área de Trabalho"'
      </div>
      <div class="button-wrapper">
        <input id="shortcut" type="checkbox" class="switch" name="shortcut" '$checked'/>
      </div>
    </li>

    <li>
      <div class="products">
        <svg viewBox="0 0 640 512">
          <path fill="currentColor" d="M224 256c70.7 0 128-57.31 128-128S294.7 0 224 0C153.3 0 96 57.31 96 128S153.3 256 224 256zM274.7 304H173.3C77.61 304 0 381.6 0 477.3C0 496.5 15.52 512 34.66 512h378.7C432.5 512 448 496.5 448 477.3C448 381.6 370.4 304 274.7 304zM616 200h-48v-48C568 138.8 557.3 128 544 128s-24 10.75-24 24v48h-48C458.8 200 448 210.8 448 224s10.75 24 24 24h48v48C520 309.3 530.8 320 544 320s24-10.75 24-24v-48h48C629.3 248 640 237.3 640 224S629.3 200 616 200z"/>
        </svg>
        '$"Perfil adicional"'
      </div>
      <div class="button-wrapper">
        <input id="addPerfilEdit" type="checkbox" class="switch" name="newperfil" '$checked_perfil'/>
      </div>
    </li>
  </ul>
</div>
<!--DETECT ICON MODAL-->
<div class="pop-up" id="detectIconEdit">
  <div class="pop-up__title">'$"Selecione o ícone preferido:"'
    <svg class="close" width="24" height="24" fill="none"
         stroke="currentColor" stroke-width="2"
         stroke-linecap="round" stroke-linejoin="round"
         class="feather feather-x-circle">
      <circle cx="12" cy="12" r="10" />
      <path d="M15 9l-6 6M9 9l6 6" />
    </svg>
  </div>
  <div id="desc">
    <div id="menu-icon"></div>
  </div>
</div>

<div class="pop-up" id="nameError">
  <div class="pop-up__subtitle">'$"Não é possível aplicar a edição sem Nome!"'</div>
  <div class="content-button-wrapper">
    <button class="content-button status-button2 close">'$"Fechar"'</button>
  </div>
</div>

<div class="pop-up" id="editError">
  <div class="pop-up__subtitle">'$"Não é possível aplicar a edição sem alterações!"'</div>
  <div class="content-button-wrapper">
    <button class="content-button status-button2 close">'$"Fechar"'</button>
  </div>
</div>

<div class="pop-up" id="editSuccess">
  <div class="pop-up__subtitle">'$"O WebApp foi editado com sucesso!"'</div>
  <div class="content-button-wrapper">
    <button class="content-button status-button2 close">'$"Fechar"'</button>
  </div>
</div>

<script type="text/javascript">

$("select").each(function(i, s){
  let getOptions = $(s).find("option");
  getOptions.sort(function(a, b) {
    return $(a).text() > $(b).text() ? 1 : -1;
  });
  $(this).html(getOptions);
});

$(function(){
  $(".pop-up#detectIconEdit .close").click(function(e){
    e.preventDefault();
    $(".pop-up#detectIconEdit").removeClass("visible");
  });

  $(".pop-up#nameError .close").click(function(e){
    e.preventDefault();
    $(".pop-up#nameError").removeClass("visible");
  });

  $(".pop-up#editError .close").click(function(e){
    e.preventDefault();
    $(".pop-up#editError").removeClass("visible");
  });

  $(".pop-up#editSuccess .close").click(function(e){
    e.preventDefault();
    $(".pop-up#editSuccess").removeClass("visible");
    document.location.reload(true);
  });

  $("#nameDeskEdit").css("border-bottom-color", "forestgreen");
  $("#nameDeskEdit").on("keyup paste search", function(){
    let checkName = $(this).val();
    if (!checkName){
      $(this).css("border-bottom-color", "");
    } else {
      $(this).css("border-bottom-color", "forestgreen");
    }
  })

  $("#loadIconEdit").click(function(e){
    e.preventDefault();
    fetch(`/execute$./change_icon.sh`)
    .then(resp => resp.text())
    .then(data => {
      if (data){
        $("#iconDeskEdit").attr("src", data);
        $("#inputIconDeskEdit").val(data);
        console.log("Change-Icon-Edit: "+data);
      } else {
        console.log("Change-Icon-Edit-Cancelled!");
      }
    });
  });

  var boxcheck = $("#addPerfilEdit").is(":checked");
  $("#browserSelectEdit").on("change", function(){
    switch (this.value){
      case "brave":
      case "com.brave.Browser":
        $("#browserEdit").attr("src", "icons/brave.svg");
        $("#addPerfilEdit").removeClass("disabled");
        if (boxcheck) {
            $("#addPerfilEdit").prop("checked", true);
        }
        break;

      case "google-chrome-stable":
      case "com.google.Chrome":
        $("#browserEdit").attr("src", "icons/chrome.svg");
        $("#addPerfilEdit").removeClass("disabled");
        if (boxcheck) {
            $("#addPerfilEdit").prop("checked", true);
        }
        break;

      case "chromium":
      case "org.chromium.Chromium":
        $("#browserEdit").attr("src", "icons/chromium.svg");
        $("#addPerfilEdit").removeClass("disabled");
        if (boxcheck) {
            $("#addPerfilEdit").prop("checked", true);
        }
        break;

      case "microsoft-edge-stable":
      case "com.microsoft.Edge":
        $("#browserEdit").attr("src", "icons/edge.svg");
        $("#addPerfilEdit").removeClass("disabled");
        if (boxcheck) {
            $("#addPerfilEdit").prop("checked", true);
        }
        break;

      case "epiphany":
      case "org.gnome.Epiphany":
        $("#browserEdit").attr("src", "icons/epiphany.svg");
        $("#addPerfilEdit").addClass("disabled");
        if (boxcheck) {
            $("#addPerfilEdit").prop("checked", false);
        }
        break;

      case "firefox":
      case "org.mozilla.firefox":
        $("#browserEdit").attr("src", "icons/firefox.svg");
        $("#addPerfilEdit").addClass("disabled");
        if (boxcheck) {
            $("#addPerfilEdit").prop("checked", false);
        }
        break;

      case "librewolf":
      case "io.gitlab.librewolf-community":
        $("#browserEdit").attr("src", "icons/librewolf.svg");
        $("#addPerfilEdit").addClass("disabled");
        if (boxcheck) {
            $("#addPerfilEdit").prop("checked", false);
        }
        break;

      case "vivaldi-stable":
        $("#browserEdit").attr("src", "icons/vivaldi.svg");
        $("#addPerfilEdit").removeClass("disabled");
        if (boxcheck) {
            $("#addPerfilEdit").prop("checked", true);
        }
        break;

      default:
          break;
    }
    console.log("Bowser-Combobox-Edit: "+this.value);
  });

  $("select#categorySelectEdit").change(function(){
    $("#imgCategoryEdit").load("icons/" + this.value + ".svg");
    console.log("Category-Edit: "+this.value)
  });

  $(".iconDetect-display-Edit").mouseover(function(){
    let srcIcon = $("#iconDeskEdit").attr("src");
    if (srcIcon !== "icons/default-webapp.svg"){
      $(".iconDetect-remove-Edit").show();
    }
  }).mouseleave(function(){
    $(".iconDetect-remove-Edit").hide();
  });

  $(".iconDetect-remove-Edit").click(function(e){
    e.preventDefault();
    $(".iconDetect-remove-Edit").hide();
    $("#iconDeskEdit").attr("src", "icons/default-webapp.svg");
    $.get("/execute$echo -n $PWD", function(cwd){
      $("#inputIconDeskEdit").val(cwd+"/icons/default-webapps.png");
      console.log("Default-Icon: "+$("#inputIconDeskEdit").val());
    });
  });

  $("#detectAllEdit").click(function(e){
    e.preventDefault();

    let url = $("#urlDeskEdit").val();
    $(".lds-ring").css("display", "inline-flex");

    fetch(`/execute$./get_title.sh.py ${url}`)
    .then(resp => resp.text())
    .then(data => {
      if (data){
        $("#nameDeskEdit").val(data);
        $("#nameDeskEdit").keyup();
      }
    });

    fetch(`/execute$./get_favicon.sh.py ${url}`)
    .then(resp => resp.text())
    .then(data => {
      if (data){
        if (/button/.test(data)){
          console.log("Multiple-Favicon");
          $(".pop-up#detectIconEdit .menu-icon").html(data)
          $(".lds-ring").css("display", "none");
          $(".pop-up#detectIconEdit").addClass("visible");
          $(".btn-img-favicon").each(function(index, el){
            $(el).click(function(e){
              e.preventDefault();
              let srcFav = $("#btn-icon-" + index + " img").attr("src");
              fetch(`/execute$./resize_favicon.sh.py ${srcFav}`)
              .then(resp => resp.text())
              .then(data => {
                $("#iconDeskEdit").attr("src", data);
                $("#inputIconDeskEdit").val(data);
                $(".pop-up#detectIconEdit").removeClass("visible");
              });
            });
          });
        } else {
          console.log("Single-Favicon");
          $("#iconDeskEdit").attr("src", data);
          $("#inputIconDeskEdit").val(data);
          $(".lds-ring").css("display", "none");
        }
      }
    });
  });

  var optionSelected = $("#browserSelectEdit").val();
  switch (optionSelected){
    case "epiphany":
    case "firefox":
    case "librewolf":
    case "org.gnome.Epiphany":
    case "org.mozilla.firefox":
    case "io.gitlab.librewolf-community":
      console.log(optionSelected);
      $("#addPerfilEdit").addClass("disabled");
      break;

    default:
      break;
  }

});

</script>' | tr -d "\t\n\r"
