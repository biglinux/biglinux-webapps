// sh_webapp-info
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

		      case "com.github.Eloston.UngoogledChromium":
		        $("#perfilAdd").removeClass("disabled");
		        $("#browser").attr("src", "icons/ungoogled.svg");
		        break;

		      case "microsoft-edge-stable":
		      case "com.microsoft.Edge":
		        $("#browserEdit").attr("src", "icons/edge.svg");
		        $("#addPerfilEdit").removeClass("disabled");
		        if (boxcheck) {
		            $("#addPerfilEdit").prop("checked", true);
		        }
		        break;

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

		      case "falkon":
		        $("#browserEdit").attr("src", "icons/falkon.svg");
		        $("#addPerfilEdit").removeClass("disabled");
		        if (boxcheck) {
		            $("#addPerfilEdit").prop("checked", true);
		        }
		        break;

		      case "opera":
		        $("#browserEdit").attr("src", "icons/opera.svg");
		        $("#addPerfilEdit").removeClass("disabled");
		        if (boxcheck) {
		            $("#addPerfilEdit").prop("checked", true);
		        }
		        break;

		      case "palemoon":
		        $("#browserEdit").attr("src", "icons/palemoon.svg");
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
		          $(".pop-up#detectIconEdit #menu-icon").html(data)
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
