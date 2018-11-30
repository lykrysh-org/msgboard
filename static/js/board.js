$(document).ready(function() {

  $('.leftbut .uploadpic').click(function() {
      $('#dark').fadeIn();
  });

  $('#dark').click(function() {
      $(this).fadeOut();
  });

  $('.edit').click(function() {
    var p = $(this).parent();
    p.parent().next(".together").hide();
    if( p.children(".count").text() == "e" ) {
      p.children(".room").hide();
      p.children(".count").text("n");
      $(this).removeClass("bbk");
    } else {
      $(this).siblings(".but").each(function() {
        $(this).removeClass("bbk");
      });
      p.children(".room").show();
      p.children(".count").text("e");
      $(this).addClass("bbk");
    }
    $(this).siblings(".room").find("input:hidden[name=_method]").val("put");
  });

  $('.delete').click(function() {
    var p = $(this).parent();
    p.parent().next(".together").hide();
    if( p.children(".count").text() == "d" ) {
      p.children(".room").hide();
      p.children(".count").text("n");
      $(this).removeClass("bbk");
    } else {
      $(this).siblings(".but").each(function() {
        $(this).removeClass("bbk");
      });
      p.children(".room").show();
      p.children(".count").text("d");
      $(this).addClass("bbk");
      $(this).siblings(".room").find("input:hidden[name=_method]").val("delete");
    }
  });

  $('.reply').click(function() {
    var p = $(this).parent();
    p.children(".room").hide();
    if( p.children(".count").text() == "r" ) {
      p.parent().next(".together").hide();
      p.children(".count").text("n");
      $(this).removeClass("bbk");
    } else {
      $(this).siblings(".but").each(function() { $(this).removeClass("bbk"); });
      p.parent().next(".together").show();
      p.children(".count").text("r");
      $(this).addClass("bbk");
    }
  });

  $('.together .footer .comment').click(function() {
    var ppp = $(this).parent().parent().parent();
    ppp.hide();
    var one = ppp.prev(".one");
    var choice = one.find(".count").text();
    one.find(".count").text("n");
    if (choice == "e") {
      one.find(".edit").removeClass("bbk");
    } else if (choice == "r") {
      one.find(".reply").removeClass("bbk");
    }
  });

  $('textarea').focus();
  $('textarea').val('');

});