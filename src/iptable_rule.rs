use std::process::Command;

// 
// Iptables uses a set of rules to determine how to filter network traffic. 
// Each rule specifies what type of traffic to filter and what action to take on matching traffic.
// 

pub fn routing_rules(client_ip: String, client_port: i32) {
// 
//This function is responsible for redirecting all traffic to the client application
//
    let rule_iptable = format!(
        "iptables -t nat -A PREROUTING -p tcp -j REDIRECT --to-ports {}",
        client_port
    );

    let status = Command::new("bash").arg("-c").arg(&rule_iptable).status();

    match status {
        Ok(exit_status) => {
            if exit_status.success() {
                println!("iptables rule added successfully.");
            } else {
                eprintln!("Error adding iptables rule: {:?}", exit_status);
            }
        }
        Err(err) => {
            eprintln!("Error executing iptables command: {:?}", err);
        }
    }

    println!("All traffic was redirected to {}:{}", client_ip, client_port);
}