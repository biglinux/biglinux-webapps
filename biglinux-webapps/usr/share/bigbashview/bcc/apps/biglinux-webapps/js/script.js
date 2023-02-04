var divs = $("div.content-section-title[id]");
if (divs.length) {
  var divsSorted = [];
  divs.sort(function (a, b) {
    return $(a).text() > $(b).text() ? 1 : -1;
  });
  for (var i = 0; i < divs.length; i++) {
    divsSorted.push($("div.content-section#" + divs[i].id));
    divsSorted.push("<br/>");
  }
  divsSorted.push($("div.pop-up#editModal"));
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
  });

  $(".dark-light").click(function (e) {
    e.preventDefault();
    $("body").toggleClass("light-mode");
  });

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

  $("#open-change-browsers").click(function () {
    var curBin = $("#open-change-browsers")
      .attr("data-bin")
      .replace(/\./g, "\\.");
    console.log("Browser-Set-Native: " + curBin);
    $("button#" + curBin).addClass("highlight");
    $(".pop-up#change-browser").addClass("visible");
  });

  $(".pop-up .close").click(function () {
    $(".pop-up").removeClass("visible");
  });

  $(".btn-img").each(function () {
    var img = $(this).children()[0];
    var src = $(img).attr("src");
    var dataBin = $(img).attr("data-bin");
    var title = $(img).attr("title");
    $(this)
      .click(function () {
        var currBin = $("#open-change-browsers").attr("data-bin");
        if (currBin === dataBin) {
          $(".pop-up#change-browser").removeClass("visible");
        } else {
          $(".pop-up#change-browser").removeClass("visible");
          $(".iconBrowser").attr("src", src);
          $("#open-change-browsers").attr("data-bin", dataBin);
          $("#browserIcon").attr("title", title);
          fetch(`/execute$./change_browser.sh ${currBin} ${dataBin}`);
        }
        console.log("Browser-Old: " + currBin, "Browser-New: " + dataBin);
      })
      .mouseover(function () {
        $("button.btn-img").removeClass("highlight");
      });
  });

  var firstOption = $("#browserSelect option").first();
  var firstValue = ((u) => u[u.length - 1])(firstOption.val().split("/"));
  $("#browser").attr("src", "icons/" + firstValue + ".svg");
  console.log("First-Browser-Combobox: " + firstValue);
  console.log("Default-Icon: " + $("#inputIconDesk").val());
  if (firstValue.toLowerCase().match(/fire(fox|dragon)|librewolf|epiphany/g))
    $("#perfilAdd").addClass("disabled");

  $("#browserSelect").on("change", function () {
    $("#browser").attr(
      "src",
      `icons/${this.querySelector("option:checked").dataset.icon}.svg`
    );
    $("#perfilAdd")[
      (this.value.toLowerCase().match(/fire(fox|dragon)|librewolf|epiphany/g)
        ? "add"
        : "remove") + "Class"
    ]("disabled");
    console.log(
      "Bowser-Combobox:",
      this.value,
      this.querySelector("option:checked").dataset.icon
    );
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

  $("#urlDesk").on("keyup paste search", function () {
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

    fetch(`/execute$./get_title.sh.py ${url}`)
      .then((resp) => resp.text())
      .then((data) => {
        if (data) {
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
            $(".pop-up#detectIcon .menu-icon").html(data);
            $(".lds-ring").css("display", "none");
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
          }
        } else {
          console.log("Favicon-Not-Found!");
          $(".lds-ring").css("display", "none");
          $(".pop-up#detectIconError").addClass("visible");
        }
      });
  });

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
        setTimeout(function () {
          $(".lds-ring").css("display", "none");
          $(".pop-up#installSuccess").addClass("visible");
        }, 2000);
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
    fetch(`/execute$./${formUrl}?${formData}`)
      .then((resp) => resp.json())
      .then((js) => {
        if (js.return) {
          console.log(js.return);
          if (js.return == 0) {
            setTimeout(function () {
              $(".lds-ring").css("display", "none");
              $(".pop-up#editSuccess").addClass("visible");
            }, 2000);
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
                  $(".pop-up#editSuccess").addClass("visible");
                }, 1000);
              });
          }, 1000);
        }
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
      console.log(data);
      $("#formEdit").html(data);
      $("#editModal").addClass("visible");
    });
}

function delDesk(filedesk) {
  console.log("Delete: " + filedesk);
  $(".lds-ring").css("display", "inline-flex");

  fetch(`/execute$./webapp-remove.sh?filedesk=${filedesk}`)
    .then((resp) => resp.text())
    .then(() => {
      setTimeout(function () {
        $(".lds-ring").css("display", "none");
      }, 2000);
      document.location.reload(true);
    });
}

$(document).keydown(function (event) {
  if (event.keyCode == 27) {
    $(".pop-up .close").click();
  }
});
