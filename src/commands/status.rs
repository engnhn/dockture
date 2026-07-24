pub async fn run_status() -> Result<(), String> {
    let docker = crate::utils::connect_docker()?;

    let options = Some(bollard::container::ListContainersOptions::<String> {
        all: true,
        ..Default::default()
    });

    let containers = docker
        .list_containers(options)
        .await
        .map_err(|e| format!("Failed to list containers: {}", e))?;

    if containers.is_empty() {
        println!("No containers found on the host.");

        let disk_path = if std::path::Path::new("/var/lib/docker").exists() {
            "/var/lib/docker"
        } else {
            "/"
        };

        if let Ok((used, total)) = crate::utils::get_disk_usage(disk_path).await {
            if total > 0 {
                let pct = (used as f64 / total as f64) * 100.0;
                let used_gb = used as f64 / (1024.0 * 1024.0 * 1024.0);
                let total_gb = total as f64 / (1024.0 * 1024.0 * 1024.0);

                let color_code = if pct >= 90.0 {
                    "\x1b[31m"
                } else if pct >= 80.0 {
                    "\x1b[33m"
                } else {
                    "\x1b[32m"
                };

                println!(
                    "Host Disk Space ({}) Usage: {}{:.1}%\x1b[0m ({:.2} GB / {:.2} GB used)",
                    disk_path, color_code, pct, used_gb, total_gb
                );
            }
        }
        return Ok(());
    }

    let col_id_w = 12;
    let mut col_name_w = 4;
    let mut col_image_w = 5;
    let mut col_state_w = 5;
    let mut col_status_w = 6;

    let formatted_containers: Vec<_> = containers
        .into_iter()
        .map(|c| {
            let id =
                c.id.as_deref()
                    .unwrap_or("")
                    .chars()
                    .take(12)
                    .collect::<String>();
            let name = c
                .names
                .as_ref()
                .and_then(|names| names.first())
                .map(|n| n.trim_start_matches('/'))
                .unwrap_or("");
            let image = c.image.as_deref().unwrap_or("");
            let state = c.state.as_deref().unwrap_or("");
            let status = c.status.as_deref().unwrap_or("");

            col_name_w = col_name_w.max(name.len());
            col_image_w = col_image_w.max(image.len());
            col_state_w = col_state_w.max(state.len());
            col_status_w = col_status_w.max(status.len());

            (
                id,
                name.to_string(),
                image.to_string(),
                state.to_string(),
                status.to_string(),
            )
        })
        .collect();

    let border_top = format!(
        "┌─{}─┬─{}─┬─{}─┬─{}─┬─{}─┐",
        "─".repeat(col_id_w),
        "─".repeat(col_name_w),
        "─".repeat(col_image_w),
        "─".repeat(col_state_w),
        "─".repeat(col_status_w)
    );
    let border_mid = format!(
        "├─{}─┼─{}─┼─{}─┼─{}─┼─{}─┤",
        "─".repeat(col_id_w),
        "─".repeat(col_name_w),
        "─".repeat(col_image_w),
        "─".repeat(col_state_w),
        "─".repeat(col_status_w)
    );
    let border_bottom = format!(
        "└─{}─┴─{}─┴─{}─┴─{}─┴─{}─┘",
        "─".repeat(col_id_w),
        "─".repeat(col_name_w),
        "─".repeat(col_image_w),
        "─".repeat(col_state_w),
        "─".repeat(col_status_w)
    );

    println!("{}", border_top);
    println!(
        "│ {:<col_id_w$} │ {:<col_name_w$} │ {:<col_image_w$} │ {:<col_state_w$} │ {:<col_status_w$} │",
        "CONTAINER ID",
        "NAME",
        "IMAGE",
        "STATE",
        "STATUS",
        col_id_w = col_id_w,
        col_name_w = col_name_w,
        col_image_w = col_image_w,
        col_state_w = col_state_w,
        col_status_w = col_status_w
    );
    println!("{}", border_mid);

    for (id, name, image, state, status) in formatted_containers {
        let state_colored = match state.as_str() {
            "running" => format!(
                "\x1b[32m{:<col_state_w$}\x1b[0m",
                state,
                col_state_w = col_state_w
            ),
            "exited" | "dead" => format!(
                "\x1b[31m{:<col_state_w$}\x1b[0m",
                state,
                col_state_w = col_state_w
            ),
            _ => format!(
                "\x1b[33m{:<col_state_w$}\x1b[0m",
                state,
                col_state_w = col_state_w
            ),
        };

        println!(
            "│ {:<col_id_w$} │ {:<col_name_w$} │ {:<col_image_w$} │ {} │ {:<col_status_w$} │",
            id,
            name,
            image,
            state_colored,
            status,
            col_id_w = col_id_w,
            col_name_w = col_name_w,
            col_image_w = col_image_w,
            col_status_w = col_status_w
        );
    }
    println!("{}", border_bottom);

    let disk_path = if std::path::Path::new("/var/lib/docker").exists() {
        "/var/lib/docker"
    } else {
        "/"
    };

    if let Ok((used, total)) = crate::utils::get_disk_usage(disk_path).await {
        if total > 0 {
            let pct = (used as f64 / total as f64) * 100.0;
            let used_gb = used as f64 / (1024.0 * 1024.0 * 1024.0);
            let total_gb = total as f64 / (1024.0 * 1024.0 * 1024.0);

            let color_code = if pct >= 90.0 {
                "\x1b[31m"
            } else if pct >= 80.0 {
                "\x1b[33m"
            } else {
                "\x1b[32m"
            };

            println!(
                "\nHost Disk Space ({}) Usage: {}{:.1}%\x1b[0m ({:.2} GB / {:.2} GB used)",
                disk_path, color_code, pct, used_gb, total_gb
            );
        }
    }

    Ok(())
}
