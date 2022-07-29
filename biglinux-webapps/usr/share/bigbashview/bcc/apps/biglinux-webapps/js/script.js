$(function () {
  var tab = $("li label");
  tab.on("click", function (event) {
    //event.preventDefault();
    //tab.removeClass("active");
    //$(this).addClass("active");
    tab_content = $(this).attr("id");
    //alert(tab_content);
    $('div[id$="tab-content"]').removeClass("active");
    $(tab_content).addClass("active");
    if(tab_content == "#add-tab-content"){
      $("#urlDesk").focus();
      $("#tab1").trigger("click");
    };
  });
});

$(function () {
 $(".menu-link").click(function () {
  $(".menu-link").removeClass("is-active");
  $(this).addClass("is-active");
 });
});

$(function () {
 $(".main-header-link").click(function () {
  $(".main-header-link").removeClass("is-active");
  $(this).addClass("is-active");
 });
});

const dropdowns = document.querySelectorAll(".dropdown");
dropdowns.forEach((dropdown) => {
 dropdown.addEventListener("click", (e) => {
  e.stopPropagation();
  dropdowns.forEach((c) => c.classList.remove("is-active"));
  dropdown.classList.add("is-active");
 });
});

$(".search-bar input")
 .focus(function () {
  $(".header").addClass("wide");
 })
 .blur(function () {
  $(".header").removeClass("wide");
 });

$(document).click(function (e) {
 var container = $(".status-button");
 var dd = $(".dropdown");
 if (!container.is(e.target) && container.has(e.target).length === 0) {
  dd.removeClass("is-active");
 }
});

$(function () {
 $(".dropdown").on("click", function (e) {
  $(".content-wrapper").addClass("overlay");
  e.stopPropagation();
 });
 $(document).on("click", function (e) {
  if ($(e.target).is(".dropdown") === false) {
   $(".content-wrapper").removeClass("overlay");
  }
 });
});

$(".dark-light").click(function (e) {
  e.preventDefault();
  $("body").toggleClass("light-mode");
});

$(".product input").click(function(e) {
  let inputId = this.id.replace(/.*\_/, "");
  let circleId = "#circle_"+inputId;
  let circleClass = $(circleId).attr("class").split(" ")[1];
  if (circleClass === "gray") {
    $(circleId).removeClass("gray").addClass("green");
  } else {
    $(circleId).removeClass("green").addClass("gray");
  }
  fetch(`/execute$./enable-disable.sh ${this.value}`);
});

$("#open-change-browsers").click(function () {
 $(".pop-up#change-browser").addClass("visible");
});

$(".pop-up .close").click(function () {
 $(".pop-up").removeClass("visible");
});

$(".btn-img").each(function(index, btn) {
  var img = $(btn).children()[0];
  var src = $(img).attr("src");
  var dataIcon = $(img).attr("data-icon");
  $(btn).click(function(){
    let curSrc = $("#browserIcon").attr("src");
    if (curSrc == src) {
      $(".pop-up#change-browser").removeClass("visible");
    } else {
      $(".pop-up#change-browser").removeClass("visible");
      $(".iconBrowser").attr("src", src);
      fetch(`/execute$./change_browser.sh ${dataIcon}`);
    }
  });
});

var firstOption = $("#browserSelect option").first();
var firstOptionValue = firstOption.val();
$("#browser").attr("src", "icons/"+firstOptionValue+".svg");
if (firstOptionValue != "firefox" && firstOptionValue != "epiphany") {
  $("#perfilAdd").show();
} else {
  $("#perfilAdd").hide();
}


$("#browserSelect").on("change",function(){
  $("#browser").attr("src", "icons/"+this.value+".svg");

  switch(this.value){
    case "epiphany":
        $("#perfilAdd").hide();
        break;

    case "firefox":
        $("#perfilAdd").hide();
        break;

    default:
        $("#perfilAdd").show();
  }
});

$("#loadIcon").click(function (e) {
  e.preventDefault();
  fetch(`/execute$./change_icon.sh`)
  .then(resp=>resp.text())
  .then(data=>{
    if(data){
      $("#iconDesk").attr("src", data);
      $("#inputIconDesk").val(data);
    }
  });
});

$("#modeTv, #loadIconChange").hide();

var invalidUrl, checkUrl, checkUrlValid;
var invalidName, checkName, checkNameValid;

$("#urlDesk").on("keyup paste search", function(){
    checkUrl = $(this).val();
    checkUrlValid = isValidURL(checkUrl);

    if(!checkUrl){
      $(this).css("border-bottom-color", "");
      $("#modeTv").hide();
      invalidUrl = true;
    } else if(!checkUrlValid){
      $(this).css("border-bottom-color", "crimson");
      $("#modeTv").hide();
      invalidUrl = true;
    } else {
      $(this).css("border-bottom-color", "forestgreen");

      if(checkUrl.match(/youtu(.be|be)/gi)){
        $("#modeTv").show();
      }

      invalidUrl = false;
    }
});

$("#nameDesk").on("keyup paste search",function(){
  checkName = $(this).val();
  checkNameValid = /\w/.test(checkName);
  if(!checkName){
    $(this).css("border-bottom-color", "");
    invalidName = true;
  }else if(!checkNameValid){
    $(this).css("border-bottom-color", "crimson");
    invalidName = true;
  } else {
    $(this).css("border-bottom-color", "forestgreen");
    invalidName = false;
  }
})

$("#detectAll").click(function (e) {
  e.preventDefault();

  let url = $("#urlDesk").val();
  if(!isValidURL(url) || !url || /\s/.test(url)){
    $(".pop-up#urlEmpty").addClass("visible");
    return;
  }

  $(".lds-ring").css("display", "inline-flex");

  fetch(`/execute$./get_title.sh.py ${url}`)
  .then(resp=>resp.text())
  .then(data=>{
    if(data){
      $("#nameDesk").val(data);
      $("#nameDesk").keyup();
    }
  });

  fetch(`/execute$./get_favicon.sh.py ${url}`)
  .then(resp=>resp.text())
  .then(data=>{
    if(data){
        if(!/\/tmp/.test(data)){
          $(".pop-up#detectIcon #menu").html(data)
          $(".lds-ring").css("display", "none");
          $(".pop-up#detectIcon").addClass("visible");
          $(".btn-img-favicon").each(function(index, el) {
            $(el).click(function (e) {
              e.preventDefault();

              $(".lds-ring").css("display", "inline-flex");
              let srcFav = $("#btn-icon-"+index+" img").attr("src");
              fetch(`/execute$./save_favicon.sh.py ${srcFav}`)
              .then(resp=>resp.text())
              .then(data=>{
                $("#iconDesk").attr("src", data);
                $("#inputIconDesk").val(data);
                $(".lds-ring").css("display", "none");
                $(".pop-up").removeClass("visible");
              });
            });
          });
        } else {
          $("#iconDesk").attr("src", data);
          $("#inputIconDesk").val(data);
          $(".lds-ring").css("display", "none");
        }
    } else{
      $(".lds-ring").css("display", "none");
      $(".pop-up#detectIconError").addClass("visible");
    }
  });
});

$("#install").click(function (e) {
  e.preventDefault();
  if(invalidUrl || invalidName || !checkUrl || !checkName){
    $(".pop-up#urlNameError").addClass("visible");
    return;
  }

  let formUrl = $("#formAdd").attr("action");
  let formData = $("#formAdd").serialize();

  fetch(`/execute$./${formUrl}?${formData}`)
  .then(resp=>resp.text())
  .then(data=>{
    if(data == 0){
      $(".lds-ring").css("display", "inline-flex");
      setTimeout(function(){
        $(".lds-ring").css("display", "none");
        $(".pop-up#installSuccess").addClass("visible");
      }, 2000);
      $("#installClose").click(function(e) {
        document.location.reload(true);
      });
    }
  });

});

$(".urlNative").mouseover(function() {
  let svg = $(this).children()[0];
  $(svg).css("display", "inline-flex");

}).mouseleave(function() {
  let svg = $(this).children()[0];
  $(svg).css("display", "none");
});

$(".btnRemove").each(function(index, element) {
  $(this).click(function (e) {
    e.preventDefault();

    $(".pop-up#remove"+index).addClass("visible");
    $("#removeYes"+index).click(function (e) {
      e.preventDefault();

      $(".pop-up#remove"+index).removeClass("visible");

      let inputRemove = $(element).children()[0];
      let filedesk = $(inputRemove).val();

      $(".lds-ring").css("display", "inline-flex");

      fetch(`/execute$./webapp-remove.sh?filedesk=${filedesk}`)
      .then(resp=>resp.text())
      .then(data=>{
        setTimeout(function(){
          $(".lds-ring").css("display", "none");
        }, 2000);
        document.location.reload(true);
      });
    });
  });
});

$("select#categorySelect").change(function(){
  $("#imgCategory").load("icons/"+this.value+".svg");
});

$(".iconDetect-display").mouseover(function(){
  let srcIcon = $("#iconDesk").attr("src");
  if(srcIcon !== "icons/default-webapp.svg"){
    $(".iconDetect-remove").show();
  }
}).mouseleave(function(){
  $(".iconDetect-remove").hide();
});

$(".iconDetect-remove").click(function(e){
  e.preventDefault();
 $(".iconDetect-remove").hide();
 $("#iconDesk").attr("src", "icons/default-webapp.svg");
 $("#inputIconDesk").val("/usr/share/bigbashview/bcc/apps/biglinux-webapps/icons/default-webapp.svg");
});


function isValidURL(string) {
  var res = string.match(/(http(s)?:\/\/.)?(www\.)?[-a-zA-Z0-9@:%._\+~#=]{2,256}(\.)?[a-z]{2,6}\b([-a-zA-Z0-9@:%_\+.~#?&//=]*)/gi);
  return (res !== null)
};
