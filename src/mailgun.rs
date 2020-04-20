use crate::user::User;

const MAILGUN_URL: &str = dotenv!("MAILGUN_URL");
const MAILGUN_API_KEY: Option<&str> = Some(dotenv!("MAILGUN_API_KEY"));

#[derive(Debug)]
pub enum MailType {
    SessionInviteReceived,
    SessionInviteAccepted,
    SessionInviteDeclined,
    GroupInviteReceived,
    GroupInviteAccepted,
    GroupInviteDeclined,
}

#[derive(Debug)]
enum ParentType {
    Session,
    Group,
}

pub fn send_mail(
    mail_type: MailType,
    user: User,
    parent_name: String,
    parent_slug: String,
    parent_owner: User,
) -> Result<u16, String> {
  println!("sending email");
    let mut to = parent_owner.email;

    let client = reqwest::blocking::Client::new();
    let from = "DnDearAll <no-reply@mg.dndearall.com>";
    let subject: &str;
    let message: &str;
    let html: String;
    let text: String;

    match mail_type {
        MailType::SessionInviteReceived => {
            to = user.email;
            subject = "You've Been Invited to a Session";
            message = "Invited you";
            html = compose_html_email(
                &parent_owner.username,
                message,
                ParentType::Session,
                &parent_name,
                &parent_slug,
            );
            text = compose_plaintext_email(
                &parent_owner.username,
                message,
                ParentType::Session,
                &parent_name,
                &parent_slug,
            );
        }
        MailType::SessionInviteAccepted => {
            subject = "Your Session Invite was Accepted";
            message = "Accepted your Invite";
            html = compose_html_email(
                &user.username,
                message,
                ParentType::Session,
                &parent_name,
                &parent_slug,
            );
            text = compose_plaintext_email(
                &user.username,
                message,
                ParentType::Session,
                &parent_name,
                &parent_slug,
            );
        }
        MailType::SessionInviteDeclined => {
            subject = "Your Session Invite was Declined";
            message = "Declined your Invite";
            html = compose_html_email(
                &user.username,
                message,
                ParentType::Session,
                &parent_name,
                &parent_slug,
            );
            text = compose_plaintext_email(
                &user.username,
                message,
                ParentType::Session,
                &parent_name,
                &parent_slug,
            );
        }
        MailType::GroupInviteReceived => {
            to = user.email;
            subject = "You've Been Invited to a Group";
            message = "Invited you";
            html = compose_html_email(
                &parent_owner.username,
                message,
                ParentType::Group,
                &parent_name,
                &parent_slug,
            );
            text = compose_plaintext_email(
                &parent_owner.username,
                message,
                ParentType::Group,
                &parent_name,
                &parent_slug,
            );
        }
        MailType::GroupInviteAccepted => {
            subject = "Your Group Invite was Accepted";
            message = "Accepted your Invite";
            html = compose_html_email(
                &user.username,
                message,
                ParentType::Group,
                &parent_name,
                &parent_slug,
            );
            text = compose_plaintext_email(
                &user.username,
                message,
                ParentType::Group,
                &parent_name,
                &parent_slug,
            );
        }
        MailType::GroupInviteDeclined => {
            subject = "Your Group Invite was Declined";
            message = "Declined your Invite";
            html = compose_html_email(
                &user.username,
                message,
                ParentType::Group,
                &parent_name,
                &parent_slug,
            );
            text = compose_plaintext_email(
                &user.username,
                message,
                ParentType::Group,
                &parent_name,
                &parent_slug,
            );
        }
    };

    client
        .post(MAILGUN_URL)
        .basic_auth("api", MAILGUN_API_KEY)
        .form(&[
            ("from", from),
            ("to", &to),
            ("subject", subject),
            ("html", &html),
            ("text", &text),
        ])
        .send()
        .map(|res| match res.status().as_u16() {
            200 => 200,
            _ => res.status().as_u16(),
        })
        .map_err(|res| format!("{:#?}", res))
}

fn compose_html_email(
    user_name: &str,
    message: &str,
    parent_type: ParentType,
    parent_name: &str,
    parent_slug: &str,
) -> String {
    let parent_group = match parent_type {
        ParentType::Session => "sessions",
        ParentType::Group => "groups",
    };
    let parent_link = format!("https://dndearall.com/#/{}/{}", parent_group, parent_slug);
    let unsubscribe_link = format!(
        "https://dndearall.com/#/unsubscribe?token={}",
        "generateUnsubscribeToken"
    );
    return format!("<!DOCTYPE html
    PUBLIC \"-//W3C//DTD XHTML 1.0 Transitional//EN\" \"http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd\">
  <html xmlns=\"http://www.w3.org/1999/xhtml\"
    style=\"font-family: 'Helvetica Neue', Helvetica, Arial, sans-serif; box-sizing: border-box; font-size: 14px; margin: 0;\">
  
  <head>
    <meta name=\"viewport\" content=\"width=device-width\" />
    <meta http-equiv=\"Content-Type\" content=\"text/html; charset=UTF-8\" />
    <title>Notification - DnDearAll</title>
  
  
    <style type=\"text/css\">
  
      img {{
        max-width: 100%;
      }}
  
      body {{
        -webkit-font-smoothing: antialiased;
        -webkit-text-size-adjust: none;
        width: 100% !important;
        height: 100%;
        line-height: 1.6em;
      }}
  
      body {{
        background-color: #f6f6f6;
      }}
  
      @media only screen and (max-width: 640px) {{
        body {{
          padding: 0 !important;
        }}
  
        h1 {{
          font-weight: 800 !important;
          margin: 20px 0 5px !important;
        }}
  
        h2 {{
          font-weight: 800 !important;
          margin: 20px 0 5px !important;
        }}
  
        h3 {{
          font-weight: 800 !important;
          margin: 20px 0 5px !important;
        }}
  
        h4 {{
          font-weight: 800 !important;
          margin: 20px 0 5px !important;
        }}
  
        h1 {{
          font-size: 22px !important;
        }}
  
        h2 {{
          font-size: 18px !important;
        }}
  
        h3 {{
          font-size: 16px !important;
        }}
  
        .container {{
          padding: 0 !important;
          width: 100% !important;
        }}
  
        .content {{
          padding: 0 !important;
        }}
  
        .content-wrap {{
          padding: 10px !important;
        }}
  
        .invoice {{
          width: 100% !important;
        }}
      }}
    </style>
  </head>
  
  <body itemscope itemtype=\"http://schema.org/EmailMessage\"
    style=\"font-family: 'Helvetica Neue',Helvetica,Arial,sans-serif; box-sizing: border-box; font-size: 14px; -webkit-font-smoothing: antialiased; -webkit-text-size-adjust: none; width: 100% !important; height: 100%; line-height: 1.6em; background-color: #f6f6f6; margin: 0;\"
    bgcolor=\"#f6f6f6\">
  
    <table class=\"body-wrap\"
      style=\"font-family: 'Helvetica Neue',Helvetica,Arial,sans-serif; box-sizing: border-box; font-size: 14px; width: 100%; background-color: #f6f6f6; margin: 0;\"
      bgcolor=\"#f6f6f6\">
      <tr
        style=\"font-family: 'Helvetica Neue',Helvetica,Arial,sans-serif; box-sizing: border-box; font-size: 14px; margin: 0;\">
        <td
          style=\"font-family: 'Helvetica Neue',Helvetica,Arial,sans-serif; box-sizing: border-box; font-size: 14px; vertical-align: top; margin: 0;\"
          valign=\"top\"></td>
        <td class=\"container\" width=\"600\"
          style=\"font-family: 'Helvetica Neue',Helvetica,Arial,sans-serif; box-sizing: border-box; font-size: 14px; vertical-align: top; display: block !important; max-width: 600px !important; clear: both !important; margin: 0 auto;\"
          valign=\"top\">
          <div class=\"content\"
            style=\"font-family: 'Helvetica Neue',Helvetica,Arial,sans-serif; box-sizing: border-box; font-size: 14px; max-width: 600px; display: block; margin: 0 auto; padding: 20px;\">
            <table class=\"main\" width=\"100%\" cellpadding=\"0\" cellspacing=\"0\"
              style=\"font-family: 'Helvetica Neue',Helvetica,Arial,sans-serif; box-sizing: border-box; font-size: 14px; border-radius: 3px; background-color: #fff; margin: 0; border: 1px solid #e9e9e9;\"
              bgcolor=\"#fff\">
              <tr
                style=\"font-family: 'Helvetica Neue',Helvetica,Arial,sans-serif; box-sizing: border-box; font-size: 14px; margin: 0;\">
                <td class=\"alert alert-warning\"
                  style=\"font-family: 'Helvetica Neue',Helvetica,Arial,sans-serif; box-sizing: border-box; font-size: 16px; vertical-align: top; color: #fff; font-weight: 500; text-align: center; border-radius: 3px 3px 0 0; background-color: #ef002b; margin: 0;\"
                  align=\"center\" bgcolor=\"#ef002b\" valign=\"top\">
                  <img
                    src=\"data:image/png;base64,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\"
                    style=\"width: 100px;
          height: 100px;\">
                </td>
              </tr>
              <tr
                style=\"font-family: 'Helvetica Neue',Helvetica,Arial,sans-serif; box-sizing: border-box; font-size: 14px; margin: 0;\">
                <td class=\"content-wrap\"
                  style=\"font-family: 'Helvetica Neue',Helvetica,Arial,sans-serif; box-sizing: border-box; font-size: 14px; vertical-align: top; margin: 0; padding: 20px;\"
                  valign=\"top\">
                  <table width=\"100%\" cellpadding=\"0\" cellspacing=\"0\"
                    style=\"font-family: 'Helvetica Neue',Helvetica,Arial,sans-serif; box-sizing: border-box; font-size: 14px; margin: 0;\">
                    <tr
                      style=\"font-family: 'Helvetica Neue',Helvetica,Arial,sans-serif; box-sizing: border-box; font-size: 14px; margin: 0;\">
                      <td class=\"content-block\"
                        style=\"font-family: 'Helvetica Neue',Helvetica,Arial,sans-serif; box-sizing: border-box; font-size: 14px; vertical-align: top; margin: 0; padding: 0 0 20px;\"
                        valign=\"top\">
                        <h1 class=\"aligncenter\"
                          style=\"font-family: Perpetua, Baskerville, Big Caslon, Palatino Linotype, Palatino, URW Palladio L, Nimbus Roman No9 L, serif; box-sizing: border-box; font-size: 32px; color: #000; line-height: 1.2em; font-weight: 500; text-align: center; margin: 0 0 0;\"
                          align=\"center\">{} has {} to {}</h1>
                      </td>
                    </tr>
                    <tr
                      style=\"font-family: 'Helvetica Neue',Helvetica,Arial,sans-serif; box-sizing: border-box; font-size: 14px; margin: 0;\">
                      <td class=\"content-block\"
                        style=\"font-family: 'Helvetica Neue',Helvetica,Arial,sans-serif; box-sizing: border-box; font-size: 14px; vertical-align: top; margin: 0; padding: 0 0 20px;\"
                        align=\"center\" valign=\"top\">
                        <a href=\"{}\" class=\"btn-primary\"
                          style=\"font-family: 'Helvetica Neue',Helvetica,Arial,sans-serif; box-sizing: border-box; font-size: 14px; color: #FFF; text-decoration: none; line-height: 2em; font-weight: bold; text-align: center; cursor: pointer; display: inline-block; border-radius: 5px; text-transform: capitalize; background-color: #9d0a0e; margin: 0; border-color: #9d0a0e; border-style: solid; border-width: 10px 20px;\">See
                          {}</a>
                      </td>
                    </tr>
                  </table>
                </td>
              </tr>
            </table>
            <div class=\"footer\"
              style=\"font-family: 'Helvetica Neue',Helvetica,Arial,sans-serif; box-sizing: border-box; font-size: 14px; width: 100%; clear: both; color: #999; margin: 0; padding: 20px;\">
              <table width=\"100%\"
                style=\"font-family: 'Helvetica Neue',Helvetica,Arial,sans-serif; box-sizing: border-box; font-size: 14px; margin: 0;\">
                <tr
                  style=\"font-family: 'Helvetica Neue',Helvetica,Arial,sans-serif; box-sizing: border-box; font-size: 14px; margin: 0;\">
                  <td class=\"aligncenter content-block\"
                    style=\"font-family: 'Helvetica Neue',Helvetica,Arial,sans-serif; box-sizing: border-box; font-size: 12px; vertical-align: top; color: #999; text-align: center; margin: 0; padding: 0 0 20px;\"
                    align=\"center\" valign=\"top\"><a href=\"{}\"
                      style=\"font-family: 'Helvetica Neue',Helvetica,Arial,sans-serif; box-sizing: border-box; font-size: 12px; color: #999; text-decoration: underline; margin: 0;\">Unsubscribe</a>
                    from these alerts.</td>
                </tr>
              </table>
            </div>
          </div>
        </td>
        <td
          style=\"font-family: 'Helvetica Neue',Helvetica,Arial,sans-serif; box-sizing: border-box; font-size: 14px; vertical-align: top; margin: 0;\"
          valign=\"top\"></td>
      </tr>
    </table>
  </body>
  
  </html>", user_name, message, parent_name, parent_link, parent_name, unsubscribe_link);
}

fn compose_plaintext_email(
    user_name: &str,
    message: &str,
    parent_type: ParentType,
    parent_name: &str,
    parent_slug: &str,
) -> String {
    let parent_group = match parent_type {
        ParentType::Session => "sessions",
        ParentType::Group => "groups",
    };
    let parent_link = format!("https://dndearall.com/#/{}/{}", parent_group, parent_slug);
    let unsubscribe_link = format!(
        "https://dndearall.com/#/unsubscribe?token={}",
        "generateUnsubscribeToken"
    );
    return format!(
        "*****************************************
    {} has {} to {}
    *****************************************
    
    See
    {} ( {} )
    
    Unsubscribe ( {} )
    from these alerts.",
        user_name, message, parent_name, parent_name, parent_link, unsubscribe_link
    );
}
