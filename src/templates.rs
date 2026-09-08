pub fn render_html_report(
    title: &str,
    badge_text: &str,
    theme_color: &str,
    theme_bg: &str,
    meta: &[(&str, String)],
    logs: Option<&str>,
) -> String {
    let mut meta_rows = String::new();
    for &(label, ref value) in meta {
        meta_rows.push_str(&format!(
            r#"<tr>
                <td class="meta-label" style="padding: 10px 0; font-size: 13px; font-weight: 600; color: #64748b; width: 35%; border-bottom: 1px solid #f1f5f9; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;">{}</td>
                <td class="meta-value" style="padding: 10px 0; font-size: 13px; color: #0f172a; font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace; border-bottom: 1px solid #f1f5f9;">{}</td>
            </tr>"#,
            label, value
        ));
    }

    let logs_html = if let Some(logs_content) = logs {
        format!(
            r#"<div class="logs-title" style="font-size: 13px; font-weight: 700; color: #334155; margin-bottom: 8px; text-transform: uppercase; letter-spacing: 0.02em; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;">Diagnostic Logs</div>
            <div class="logs-box" style="background-color: #f8fafc; border: 1px solid #cbd5e1; border-radius: 8px; padding: 14px; font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace; font-size: 12px; color: #334155; white-space: pre-wrap; word-break: break-all; line-height: 1.5;">{}</div>"#,
            logs_content
        )
    } else {
        String::new()
    };

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>{title}</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
            background-color: #f8fafc;
            margin: 0;
            padding: 24px;
            color: #334155;
        }}
        .card-table {{
            max-width: 600px;
            margin: 0 auto;
            background-color: #ffffff;
            border-radius: 12px;
            border: 1px solid #e2e8f0;
            border-collapse: separate;
            overflow: hidden;
            box-shadow: 0 4px 6px -1px rgba(0,0,0,0.05), 0 2px 4px -1px rgba(0,0,0,0.03);
        }}
        .header-section {{
            background-color: {theme_bg};
            border-left: 6px solid {theme_color};
            padding: 28px 24px;
            border-bottom: 1px solid #e2e8f0;
        }}
        .logo {{
            font-size: 15px;
            font-weight: 800;
            color: #0f766e;
            letter-spacing: -0.02em;
            margin: 0 0 6px 0;
            font-family: ui-sans-serif, system-ui, sans-serif;
        }}
        .logo span {{
            color: #0d9488;
            font-weight: 400;
        }}
        .header-title {{
            font-size: 20px;
            font-weight: 700;
            color: #0f172a;
            margin: 0 0 12px 0;
        }}
        .badge {{
            display: inline-block;
            font-size: 11px;
            font-weight: 700;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            background-color: #ffffff;
            color: {theme_color};
            border: 1.5px solid {theme_color};
            padding: 3px 8px;
            border-radius: 6px;
        }}
        .body-section {{
            padding: 24px;
        }}
        .meta-table {{
            width: 100%;
            border-collapse: collapse;
            margin-bottom: 20px;
        }}
        .meta-table tr {{
            border-bottom: 1px solid #f1f5f9;
        }}
        .meta-table tr:last-child {{
            border-bottom: none;
        }}
        .meta-label {{
            padding: 10px 0;
            font-size: 13px;
            font-weight: 600;
            color: #64748b;
            width: 35%;
        }}
        .meta-value {{
            padding: 10px 0;
            font-size: 13px;
            color: #0f172a;
            font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
        }}
        .logs-title {{
            font-size: 13px;
            font-weight: 700;
            color: #334155;
            margin-bottom: 8px;
            text-transform: uppercase;
            letter-spacing: 0.02em;
        }}
        .logs-box {{
            background-color: #f8fafc;
            border: 1px solid #cbd5e1;
            border-radius: 8px;
            padding: 14px;
            font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
            font-size: 12px;
            color: #334155;
            white-space: pre-wrap;
            word-break: break-all;
            line-height: 1.5;
        }}
        .footer-section {{
            background-color: #f8fafc;
            border-top: 1px solid #e2e8f0;
            padding: 16px 24px;
            font-size: 11px;
            color: #94a3b8;
            text-align: center;
        }}
    </style>
</head>
<body>
    <table width="100%" border="0" cellspacing="0" cellpadding="0">
        <tr>
            <td align="center" style="padding: 20px 0;">
                <table class="card-table" width="100%" border="0" cellspacing="0" cellpadding="0" style="max-width: 600px; margin: 0 auto; background-color: #ffffff; border-radius: 12px; border: 1px solid #e2e8f0; border-collapse: separate; overflow: hidden; box-shadow: 0 4px 6px -1px rgba(0,0,0,0.05), 0 2px 4px -1px rgba(0,0,0,0.03);">
                    <tr>
                        <td class="header-section" style="background-color: {theme_bg}; border-left: 6px solid {theme_color}; padding: 28px 24px; border-bottom: 1px solid #e2e8f0;">
                            <div class="logo" style="font-size: 15px; font-weight: 800; color: #0f766e; letter-spacing: -0.02em; margin: 0 0 6px 0; font-family: ui-sans-serif, system-ui, sans-serif;">dock<span style="color: #0d9488; font-weight: 400;">ture</span></div>
                            <h1 class="header-title" style="font-size: 20px; font-weight: 700; color: #0f172a; margin: 0 0 12px 0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;">{title}</h1>
                            <div class="badge" style="display: inline-block; font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.05em; background-color: #ffffff; color: {theme_color}; border: 1.5px solid {theme_color}; padding: 3px 8px; border-radius: 6px; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;">{badge_text}</div>
                        </td>
                    </tr>
                    <tr>
                        <td class="body-section" style="padding: 24px;">
                            <table class="meta-table" style="width: 100%; border-collapse: collapse; margin-bottom: 20px;">
                                {meta_rows}
                            </table>
                            {logs_html}
                        </td>
                    </tr>
                    <tr>
                        <td class="footer-section" style="background-color: #f8fafc; border-top: 1px solid #e2e8f0; padding: 16px 24px; font-size: 11px; color: #94a3b8; text-align: center; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;">
                            This alert was automatically generated by dockture monitor daemon.
                        </td>
                    </tr>
                </table>
            </td>
        </tr>
    </table>
</body>
</html>"#,
        title = title,
        theme_bg = theme_bg,
        theme_color = theme_color,
        badge_text = badge_text,
        meta_rows = meta_rows,
        logs_html = logs_html
    )
}

pub fn render_daily_report_html(
    report_date: &str,
    total_monitored: usize,
    crashes: u32,
    health_failures: u32,
    log_matches: u32,
    resource_warnings: u32,
    anomalies: u32,
    disk_usage_info: &str,
    container_summary_rows: &[(&str, &str, u32)], // (Name, Status/State, AlertCount)
) -> String {
    let mut rows = String::new();
    for &(name, status, alerts) in container_summary_rows {
        let status_color = if status == "running" {
            "#10b981"
        } else if status == "unhealthy" {
            "#ef4444"
        } else {
            "#f59e0b"
        };
        rows.push_str(&format!(
            r#"<tr>
                <td style="padding: 10px; font-size: 13px; font-weight: 600; color: #0f172a; border-bottom: 1px solid #f1f5f9; font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;">{}</td>
                <td style="padding: 10px; font-size: 12px; font-weight: 700; color: {}; border-bottom: 1px solid #f1f5f9; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;">{}</td>
                <td style="padding: 10px; font-size: 13px; color: #334155; border-bottom: 1px solid #f1f5f9; text-align: center; font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;">{}</td>
            </tr>"#,
            name, status_color, status.to_uppercase(), alerts
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Dockture Daily System Report - {date}</title>
</head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Arial, sans-serif; background-color: #f8fafc; margin: 0; padding: 24px; color: #334155;">
    <table width="100%" border="0" cellspacing="0" cellpadding="0">
        <tr>
            <td align="center" style="padding: 20px 0;">
                <table width="100%" border="0" cellspacing="0" cellpadding="0" style="max-width: 600px; margin: 0 auto; background-color: #ffffff; border-radius: 12px; border: 1px solid #e2e8f0; border-collapse: separate; overflow: hidden; box-shadow: 0 4px 6px -1px rgba(0,0,0,0.05);">
                    <tr>
                        <td style="background-color: #f0fdf4; border-left: 6px solid #10b981; padding: 28px 24px; border-bottom: 1px solid #e2e8f0;">
                            <div style="font-size: 15px; font-weight: 800; color: #0f766e; margin: 0 0 6px 0;">dock<span style="color: #0d9488; font-weight: 400;">ture</span></div>
                            <h1 style="font-size: 20px; font-weight: 700; color: #0f172a; margin: 0 0 8px 0;">Daily Health Summary Report</h1>
                            <div style="font-size: 12px; font-weight: 600; color: #047857;">Report Date: {date}</div>
                        </td>
                    </tr>
                    <tr>
                        <td style="padding: 24px;">
                            <h3 style="font-size: 14px; font-weight: 700; color: #0f172a; margin-top: 0; margin-bottom: 14px; text-transform: uppercase; letter-spacing: 0.03em;">24-Hour System Overview</h3>
                            <table width="100%" border="0" cellspacing="0" cellpadding="0" style="margin-bottom: 20px; text-align: center;">
                                <tr>
                                    <td width="33%" style="background-color: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 12px;">
                                        <div style="font-size: 11px; font-weight: 700; color: #64748b; text-transform: uppercase;">Monitored</div>
                                        <div style="font-size: 22px; font-weight: 800; color: #0f172a; margin-top: 4px;">{total_monitored}</div>
                                    </td>
                                    <td width="33%" style="background-color: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 12px;">
                                        <div style="font-size: 11px; font-weight: 700; color: #64748b; text-transform: uppercase;">Crashes/OOM</div>
                                        <div style="font-size: 22px; font-weight: 800; color: {crashes_color}; margin-top: 4px;">{crashes}</div>
                                    </td>
                                    <td width="33%" style="background-color: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 12px;">
                                        <div style="font-size: 11px; font-weight: 700; color: #64748b; text-transform: uppercase;">Anomalies</div>
                                        <div style="font-size: 22px; font-weight: 800; color: {anomalies_color}; margin-top: 4px;">{anomalies}</div>
                                    </td>
                                </tr>
                            </table>

                            <table width="100%" border="0" cellspacing="0" cellpadding="0" style="margin-bottom: 24px; font-size: 13px; color: #475569; border-collapse: collapse;">
                                <tr style="border-bottom: 1px solid #f1f5f9;">
                                    <td style="padding: 8px 0; font-weight: 600; color: #64748b;">Health Status Failures:</td>
                                    <td style="padding: 8px 0; font-weight: 700; text-align: right; color: #0f172a;">{health_failures}</td>
                                </tr>
                                <tr style="border-bottom: 1px solid #f1f5f9;">
                                    <td style="padding: 8px 0; font-weight: 600; color: #64748b;">Log Error Keyword Matches:</td>
                                    <td style="padding: 8px 0; font-weight: 700; text-align: right; color: #0f172a;">{log_matches}</td>
                                </tr>
                                <tr style="border-bottom: 1px solid #f1f5f9;">
                                    <td style="padding: 8px 0; font-weight: 600; color: #64748b;">High CPU/Memory Warnings:</td>
                                    <td style="padding: 8px 0; font-weight: 700; text-align: right; color: #0f172a;">{resource_warnings}</td>
                                </tr>
                                <tr>
                                    <td style="padding: 8px 0; font-weight: 600; color: #64748b;">Host Storage Utilization:</td>
                                    <td style="padding: 8px 0; font-weight: 700; text-align: right; color: #0f172a;">{disk_usage_info}</td>
                                </tr>
                            </table>

                            <h3 style="font-size: 14px; font-weight: 700; color: #0f172a; margin-top: 0; margin-bottom: 12px; text-transform: uppercase; letter-spacing: 0.03em;">Container Status Details</h3>
                            <table width="100%" border="0" cellspacing="0" cellpadding="0" style="border-collapse: collapse; width: 100%;">
                                <thead>
                                    <tr style="background-color: #f8fafc; border-bottom: 2px solid #e2e8f0;">
                                        <th style="padding: 10px; font-size: 11px; font-weight: 700; color: #64748b; text-transform: uppercase; text-align: left;">Container</th>
                                        <th style="padding: 10px; font-size: 11px; font-weight: 700; color: #64748b; text-transform: uppercase; text-align: left;">Status</th>
                                        <th style="padding: 10px; font-size: 11px; font-weight: 700; color: #64748b; text-transform: uppercase; text-align: center;">24h Alerts</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {container_rows}
                                </tbody>
                            </table>
                        </td>
                    </tr>
                    <tr>
                        <td style="background-color: #f8fafc; border-top: 1px solid #e2e8f0; padding: 16px 24px; font-size: 11px; color: #94a3b8; text-align: center;">
                            Automated daily system summary generated by dockture monitor daemon.
                        </td>
                    </tr>
                </table>
            </td>
        </tr>
    </table>
</body>
</html>"#,
        date = report_date,
        total_monitored = total_monitored,
        crashes = crashes,
        crashes_color = if crashes > 0 { "#ef4444" } else { "#10b981" },
        anomalies = anomalies,
        anomalies_color = if anomalies > 0 { "#f59e0b" } else { "#10b981" },
        health_failures = health_failures,
        log_matches = log_matches,
        resource_warnings = resource_warnings,
        disk_usage_info = disk_usage_info,
        container_rows = rows
    )
}

