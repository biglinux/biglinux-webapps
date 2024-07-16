var divs = $("div.content-section-title[id]");
if (divs.length) {
  var divsSorted = [];
  divsSorted.push($(".menu"));
  divs.sort(function (a, b) {
    return $(a).text() > $(b).text() ? 1 : -1;
  });
  for (var i = 0; i < divs.length; i++) {
    divsSorted.push($("div.content-section#" + divs[i].id));
    divsSorted.push("<br/>");
  }
  divsSorted.push($("div.pop-up#editModal"));
  divsSorted.push($("div.pop-up#backupModal"));
  divsSorted.push($("div.pop-up#restoreModal"));
  divsSorted.push($("div.pop-up#removeAllModal"));
  $("#list-tab-content").html(divsSorted);
}

$("div.content-section-title[id]").each(function (i, div) {
  let countLi = $("#" + div.id + " li").length;
  $("span#" + div.id).text(countLi);
});

$("ul[id]").each(function (i, d) {
  let getDivs = $(d).find("li");
  getDivs.sort(function (a, b) {
    return $(a).text() > $(b).text() ? 1 : -1;
  });
  $(this).html(getDivs);
});

$("select").each(function (i, s) {
  let getOptions = $(s).find("option");
  getOptions.sort(function (a, b) {
    return $(a).text() > $(b).text() ? 1 : -1;
  });
  $(this).html(getOptions);
});

function wrapper_browser(name_exec) {
  switch (name_exec) {
    case "brave":
    case "com.brave.Browser":
      $("#perfilAdd").removeClass("disabled");
      $("#browser").attr("src", "icons/brave.svg");
      break;

    case "google-chrome-stable":
    case "com.google.Chrome":
      $("#perfilAdd").removeClass("disabled");
      $("#browser").attr("src", "icons/chrome.svg");
      break;

    case "chromium":
    case "org.chromium.Chromium":
      $("#perfilAdd").removeClass("disabled");
      $("#browser").attr("src", "icons/chromium.svg");
      break;

    case "com.github.Eloston.UngoogledChromium":
      $("#perfilAdd").removeClass("disabled");
      $("#browser").attr("src", "icons/ungoogled.svg");
      break;

    case "microsoft-edge-stable":
    case "com.microsoft.Edge":
      $("#perfilAdd").removeClass("disabled");
      $("#browser").attr("src", "icons/edge.svg");
      break;

    case "epiphany":
    case "org.gnome.Epiphany":
      $("#perfilAdd").addClass("disabled");
      $("#browser").attr("src", "icons/epiphany.svg");
      break;

    case "firefox":
    case "org.mozilla.firefox":
      $("#perfilAdd").addClass("disabled");
      $("#browser").attr("src", "icons/firefox.svg");
      break;

    case "librewolf":
    case "io.gitlab.librewolf-community":
      $("#perfilAdd").addClass("disabled");
      $("#browser").attr("src", "icons/librewolf.svg");
      break;

    case "vivaldi-stable":
      $("#perfilAdd").removeClass("disabled");
      $("#browser").attr("src", "icons/vivaldi.svg");
      break;

    case "falkon":
      $("#perfilAdd").removeClass("disabled");
      $("#browser").attr("src", "icons/falkon.svg");
      break;

    case "opera":
      $("#perfilAdd").removeClass("disabled");
      $("#browser").attr("src", "icons/opera.svg");
      break;

    case "palemoon":
      $("#perfilAdd").removeClass("disabled");
      $("#browser").attr("src", "icons/palemoon.svg");
      break;

    default:
      break;
  }
}

$(function () {
  var tab = $("li label");
  var checkUrl;
  var checkName;

  tab.on("click", function () {
    tab_content = $(this).attr("id");
    $('div[id$="tab-content"]').removeClass("active");
    $(tab_content).addClass("active");
    if (tab_content == "#add-tab-content") {
      $("#urlDesk").focus();
      $("#tab1").trigger("click");
    }
    if (tab_content == "#list-tab-content") {
      $("#tab2").trigger("click");
    }
    // insert
    if (tab_content == "#webappsbig-tab-content") {
      $("#tab3").trigger("click");
    }
    if (tab_content == "#about-tab-content") {
      $("#tab4").trigger("click");
    }
    // insert
  });

  //  $(".dark-light").click(function (e) {
  //    e.preventDefault();
  //    $("body").toggleClass("light-mode");
  //  });

  // Vilmar Catafesta, <vcatafesta@gmail.com> ter 04 jun 2024 07:27:38 -04
  const toggleButton = document.querySelector(".dark-light");
  toggleButton.addEventListener("click", () => {
    let state = document.body.classList.contains("light-mode");
    _run("sh_webapp_setbgcolor " + state);
    console.log("light-mode =:", state);
    document.body.classList.toggle("light-mode");
    state = document.body.classList.contains("light-mode");
    console.log("light-mode =:", state);
  });
  // Vilmar Catafesta, <vcatafesta@gmail.com> ter 04 jun 2024 07:27:38 -04

  $(".product input").click(function () {
    let inputId = this.id.replace(/.*\_/, "");
    let circleId = "#circle_" + inputId;
    let linkId = "#link_" + inputId;
    let circleClass = $(circleId).attr("class").split(" ")[1];
    if (circleClass === "gray") {
      $(circleId).removeClass("gray").addClass("green");
      $(linkId).removeClass("disabled");
    } else {
      $(circleId).removeClass("green").addClass("gray");
      $(linkId).addClass("disabled");
    }
    let browserBin = $("#open-change-browsers").attr("data-bin");
    fetch(`/execute$./enable-disable.sh ${this.value} ${browserBin}`);
    console.log("Filedesk: " + this.value, "Browser cmd: " + browserBin);
  });

	// ################################################################################################################################################
	// Change Browser
	// ################################################################################################################################################
  $("#open-change-browsers").click(function () {
    var curBin = $("#open-change-browsers").attr("data-bin").replace(/\./g, "\\.");
    console.log("Browser-Set-Native: " + curBin);
    $("button#" + curBin).addClass("highlight");
    $(".pop-up#change-browser").addClass("visible");
  });

  $(".pop-up .close").click(function () {
    $(".pop-up").removeClass("visible");
  });

  $(".btn-img").each(function () {
    // Para cada elemento com a classe .btn-img
    var img = $(this).children()[0];				// Seleciona o primeiro filho do elemento .btn-img
    var src = $(img).attr("src");						// Obtém o atributo src da imagem
    var dataBin = $(img).attr("data-bin");	// Obtém o atributo data-bin da imagem
    var title = $(img).attr("title");				// Obtém o atributo title da imagem

    // Ao clicar no elemento .btn-img
    $(this)
      .click(function () {
        var currBin = $("#open-change-browsers").attr("data-bin");			// Obtém o atributo data-bin de #open-change-browsers

        if (currBin === dataBin) {
	        console.log("Nada mudou")
          // Se o data-bin atual for igual ao data-bin da imagem clicada
          $(".pop-up#change-browser").removeClass("visible");						// Remove a classe 'visible' da pop-up de mudança de navegador
        } else {
          // Caso contrário
	        console.log("Alterando navegador")
          $(".pop-up#change-browser").removeClass("visible");						// Remove a classe 'visible' da pop-up de mudança de navegador

//        $(".iconBrowser").attr("src", src);														// Atualiza o atributo src do elemento com a classe .iconBrowser
					var countCheck = 0;
					$(".switch").each(function() {
		        countCheck++;
		        var isChecked = $(this).is(":checked");
		        var id = $(this).attr("id"); // Pega o id do switch
		        if (isChecked) {
		          console.log('Imagem já tem o src correto :', $("#imgsrc" + id).attr("src"));
		        } else {
		          $("#imgsrc" + id).attr("src", src);
							console.log('Imagem atualizada para :', src);
		        }
		      });

          $("#open-change-browsers").attr("data-bin", dataBin);					// Atualiza o atributo data-bin de #open-change-browsers
          $("#browserIcon").attr("src", src);
          $("#browserIcon").attr("title", title);												// Atualiza o atributo title de #browserIcon
          fetch(`/execute$./change_browser.sh ${currBin} ${dataBin}`);	// Executa um fetch para mudar o navegador com os bins atuais e novos
	        console.log("Navegador Antigo: " + currBin);
					console.log("Navegador Novo  : " + dataBin);
        }
      })
      .mouseover(function () {
        $("button.btn-img").removeClass("highlight");										// Ao passar o mouse sobre, remove a classe 'highlight' de todos os botões .btn-img
      });
  });
	// ################################################################################################################################################
	// Change Browser
	// ################################################################################################################################################

  var firstOption = $("#browserSelect option").first();
  var firstValue = firstOption.val();
  console.log("First-Browser-Combobox: " + firstValue);
  wrapper_browser(firstValue);

  console.log("Default-Icon: " + $("#inputIconDesk").val());

  $("#browserSelect").on("change", function () {
    wrapper_browser(this.value);
    console.log("Bowser-Combobox: " + this.value);
  });

  $("#loadIcon").click(function (e) {
    e.preventDefault();
    fetch(`/execute$./change_icon.sh`)
      .then((resp) => resp.text())
      .then((data) => {
        if (data) {
          $("#iconDesk").attr("src", data);
          $("#inputIconDesk").val(data);
          console.log("Change-Icon: " + data);
        } else {
          console.log("Change-Icon-Cancelled!");
        }
      });
  });

  $("#urlDesk").on("keyup paste search blur", function () {
    checkUrl = $(this).val();

    if (!checkUrl) {
      $(this).css("border-bottom-color", "");
    } else if (/\s/.test(checkUrl)) {
      $(this).css("border-bottom-color", "crimson");
    } else {
      $(this).css("border-bottom-color", "forestgreen");
    }
  });

  $("#nameDesk").on("keyup paste search", function () {
    checkName = $(this).val();

    if (!checkName) {
      $(this).css("border-bottom-color", "");
    } else {
      $(this).css("border-bottom-color", "forestgreen");
    }
  });

  $("#detectAll").click(function (e) {
    e.preventDefault();

    let url = $("#urlDesk").val();
    if (!url || /\s/.test(url)) {
      $(".pop-up#urlEmpty").addClass("visible");
      return;
    }

    $(".lds-ring").css("display", "inline-flex");
    $("#text-loading-name").show();

    fetch(`/execute$./get_title.sh.py ${url}`)
      .then((resp) => resp.text())
      .then((data) => {
        if (data) {
          $("#text-loading-name").hide();
          $("#text-loading-icon").show();
          $("#nameDesk").val(data);
          $("#nameDesk").keyup();
        } else {
          console.log("Title-Not-Found!");
        }
      });

    fetch(`/execute$./get_favicon.sh.py ${url}`)
      .then((resp) => resp.text())
      .then((data) => {
        if (data) {
          if (/button/.test(data)) {
            console.log("Multiple-Favicon");
            $(".pop-up#detectIcon #menu-icon").html(data);
            $(".lds-ring").css("display", "none");
            $("#text-loading-icon").hide();
            $(".pop-up#detectIcon").addClass("visible");
            $(".btn-img-favicon").each(function (index, el) {
              $(el).click(function (e) {
                e.preventDefault();
                let srcFav = $("#btn-icon-" + index + " img").attr("src");
                fetch(`/execute$./resize_favicon.sh.py ${srcFav}`)
                  .then((resp) => resp.text())
                  .then((data) => {
                    $("#iconDesk").attr("src", data);
                    $("#inputIconDesk").val(data);
                    $(".pop-up#detectIcon").removeClass("visible");
                  });
              });
            });
          } else {
            console.log("Single-Favicon");
            $("#iconDesk").attr("src", data);
            $("#inputIconDesk").val(data);
            $(".lds-ring").css("display", "none");
            $("#text-loading-icon").hide();
          }
        } else {
          console.log("Favicon-Not-Found!");
          $(".lds-ring").css("display", "none");
          $("#text-loading-icon").hide();
          $(".pop-up#detectIconError").addClass("visible");
        }
      });
  });

  $("#cancel").click(() => {
    $("#urlDesk").css("border-bottom-color", "");
    $("#iconDesk").attr("src", "icons/default-webapp.svg");
    $.get(`/execute$echo -n "$PWD"`, function (cwd) {
      $("#inputIconDesk").val(cwd + "/icons/default-webapps.png");
    });
    $("li label#" + $.escapeSelector("#list-tab-content")).click();
  });

  $("#add").click(() => {
    $("li label#" + $.escapeSelector("#add-tab-content")).click();
  });

  // insert
  $("#add").click(() => {
    $("li label#" + $.escapeSelector("#about-tab-content")).click();
  });
  // insert

  $("#install").click(function (e) {
    e.preventDefault();

    if (!checkUrl || /\s/.test(checkUrl) || !checkName) {
      $(".pop-up#urlNameError").addClass("visible");
      return;
    }

    let formUrl = $("#formAdd").attr("action");
    let formData = $("#formAdd").serialize();

    fetch(`/execute$./${formUrl}?${formData}`)
      .then((resp) => resp.text())
      .then(() => {
        $(".lds-ring").css("display", "inline-flex");
        $("#text-loading-add").show();
        setTimeout(function () {
          $(".lds-ring").css("display", "none");
          $("#text-loading-add").hide();
          //        $(".pop-up#installSuccess").addClass("visible");
          // insert
          $("#installClose").click();
          // insert
        }, 3000);
        $("#installClose").click(function () {
          document.location.reload(true);
        });
      });
  });

  $(".urlNative, .urlCustom")
    .mouseover(function () {
      let svg = $(this).children()[0];
      $(svg).css("display", "inline-flex");
    })
    .mouseleave(function () {
      let svg = $(this).children()[0];
      $(svg).css("display", "none");
    });

  $("select#categorySelect").change(function () {
    $("#imgCategory").load("icons/" + this.value + ".svg");
    console.log("Category: " + this.value);
  });

  $(".iconDetect-display")
    .mouseover(function () {
      let srcIcon = $("#iconDesk").attr("src");
      if (srcIcon !== "icons/default-webapp.svg") {
        $(".iconDetect-remove").show();
      }
    })
    .mouseleave(function () {
      $(".iconDetect-remove").hide();
    });

  $(".iconDetect-remove").click(function (e) {
    e.preventDefault();
    $(".iconDetect-remove").hide();
    $("#iconDesk").attr("src", "icons/default-webapp.svg");
    $.get(`/execute$echo -n "$PWD"`, function (cwd) {
      $("#inputIconDesk").val(cwd + "/icons/default-webapps.png");
      console.log("Default-Icon: " + $("#inputIconDesk").val());
    });
  });

  $("#submitEdit").click(function (e) {
    e.preventDefault();
    let formUrl = $("#editForm").attr("action");
    let formData = $("#editForm").serialize();

    if (!$("#nameDeskEdit").val()) {
      $(".pop-up#nameError").addClass("visible");
      return;
    }

    $(".lds-ring").css("display", "inline-flex");
    $("#text-loading-edit").show();
    fetch(`/execute$./${formUrl}?${formData}`)
      .then((resp) => resp.json())
      .then((js) => {
        if (js.return) {
          console.log(js.return);
          if (js.return == 0) {
            setTimeout(function () {
              $(".lds-ring").css("display", "none");
              $("#text-loading-edit").hide();
              $(".pop-up#editSuccess").addClass("visible");
            }, 3000);
          } else {
            $(".pop-up#editError").addClass("visible");
            return;
          }
        } else {
          let browser = js.browser;
          let category = js.category;
          let filedesk = js.filedesk;
          let icondesk = js.icondesk;
          let namedesk = js.namedesk;
          let newperfil = js.newperfil;
          let shortcut = js.shortcut;
          let urldesk = js.urldesk;
          console.clear();
          console.log(js);
          fetch(`/execute$./webapp-remove.sh?filedesk=${filedesk}`);
          setTimeout(function () {
            fetch(
              `/execute$./webapp-install.sh?browser=${browser}&category=${category}&icondesk=${icondesk}&namedesk=${namedesk}&newperfil=${newperfil}&shortcut=${shortcut}&urldesk=${urldesk}`
            )
              .then((r) => r.text())
              .then(() => {
                setTimeout(function () {
                  $(".lds-ring").css("display", "none");
                  $("#text-loading-edit").hide();
                  $(".pop-up#editSuccess").addClass("visible");
                }, 1500);
              });
          }, 1500);
        }
      });
  });

  // insert
  $("#ativar").click(function (e) {
    e.preventDefault();
    $(".lds-ring").css("display", "inline-flex");
    $("#text-loading-ativar").show();

    $.get("/execute$./ativar.sh", function (data) {
      $(".lds-ring").css("display", "none");
      document.location.reload(true);
      $("#text-loading-ativar").hide();
    });
  });

  $("#desativar").click(function (e) {
    e.preventDefault();
    $(".lds-ring").css("display", "inline-flex");
    $("#text-loading-desativar").show();

    $.get("/execute$./desativar.sh", function (data) {
      $(".lds-ring").css("display", "none");
      document.location.reload(true);
      $("#text-loading-desativar").hide();
    });
  });
  // insert

  $("#backup").click(function (e) {
    e.preventDefault();
    $(".lds-ring").css("display", "inline-flex");
    $("#text-loading-bkp").show();

    $.get("/execute$./backup.sh", function (data) {
      $(".lds-ring").css("display", "none");
      $("#text-loading-bkp").hide();
      if (data) {
        console.log(data);
        $("#backupPath").text(data);
        $(".pop-up#backupModal").addClass("visible");
      }
    });
  });

  $("#restore").click(function (e) {
    e.preventDefault();
    $(".lds-ring").css("display", "inline-flex");
    $("#text-loading-restore").show();
    $.get("/execute$./restore.sh", function (resp) {
      $(".lds-ring").css("display", "none");
      $("#text-loading-restore").hide();
      if (resp) {
        $("#restoreModal").addClass("visible");
        $("#closeRestore").click(function () {
          location.reload(true);
        });
      }
    });
  });

  $("#del-all").click(function (e) {
    e.preventDefault();
    $("#removeAllModal").addClass("visible");
    $("#removeAllYes").click(function (e) {
      e.preventDefault();
      $(".lds-ring").css("display", "inline-flex");
      $("#text-loading-del-all").show();
      fetch(`/execute$./webapp-remove-all.sh`)
        .then((resp) => resp.text())
        .then((data) => {
          console.log(data);
          location.reload(true);
        });
    });
  });
});

function delOpen(id) {
  $(".pop-up#" + id).addClass("visible");
}

function editOpen(filedesk) {
  console.log("Edit: " + filedesk);
  fetch(`/execute$./webapp-info.sh?filedesk=${filedesk}`)
    .then((resp) => resp.text())
    .then((data) => {
      $("#formEdit").html(data);
      $("#editModal").addClass("visible");
    });
}

function delDesk(filedesk) {
  console.log("Delete: " + filedesk);
  $(".lds-ring").css("display", "inline-flex");
  $("#text-loading-del").show();

  fetch(`/execute$./webapp-remove.sh?filedesk=${filedesk}`)
    .then((resp) => resp.text())
    .then(() => {
      setTimeout(function () {
        $(".lds-ring").css("display", "none");
        $("#text-loading-del").hide();
      }, 3000);
      document.location.reload(true);
    });
}

$(document).keydown(function (event) {
  if (event.keyCode == 27) {
    $(".pop-up .close").click();
  }
});

$(document).click(function (e) {
  if (e.target.className === "dropdown") {
    $(".dropdown").addClass("is-active");
  } else {
    $(".dropdown").removeClass("is-active");
  }
});
