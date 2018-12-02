$(document).ready(function() {

  $('.leftbut .uploadpic').click(function() {
      $('#dark').fadeIn();
  });

  $('#dark').click(function() {
      $(this).fadeOut();
  });

  $('.topform').bind('submit',function(e){
    var trouble = false;
    var msg = '';
    if ($.trim($(this).find("#description").val()) === '') { msg += 'Any text? '; trouble = true; };
    if ($.trim($(this).find("#whosent").val()) === '') { msg += 'Your name? '; trouble = true; };
    if ($.trim($(this).find("#secret").val()) === '') { msg += 'Secret key? '; trouble = true; };
    if (trouble) { 
      $('#bugbug').text(msg);
      e.preventDefault(); 
    };
  });

  $('.replyform').bind('submit',function(e){
    var trouble = false;
    var msg = '';
    if ($.trim($(this).find("#description").val()) === '') { msg += 'Any text? '; trouble = true; };
    if ($.trim($(this).find("#whosent").val()) === '') { msg += 'Your name? '; trouble = true; };
    if ($.trim($(this).find("#secret").val()) === '') { msg += 'Secret key? '; trouble = true; };
    if (trouble) { 
      $('#bugbug').text(msg);
      e.preventDefault(); 
    } else {
        var ppp = $(this).parent();
        ppp.hide();
        var one = ppp.prev(".one");
        var choice = one.find(".count").text();
        one.find(".count").text("n");
        if (choice == "e") {
          one.find(".edit").removeClass("bbk");
        } else if (choice == "r") {
          one.find(".reply").removeClass("bbk");
        }
    } 
  });

  $('.editform').on('reset',function(){  
  });

  $('.editform').bind('submit',function(){
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

  $(document).mousemove(function(e){
    if (! $('#bugbug').is(':empty')) {
      setTimeout(function(){
        $('#bugbug').text(''); 
      }, 800);
    }
  });

  $('.topform textarea').focus().val('');
  $('.replyform textarea').focus().val('');

});