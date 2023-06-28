use iptables::Chain;
use iptables::IpVersion;
use iptables::Table;

pub fn routing_rules(client_ip: &str, client_port: i32) {

    //Create a new iptable
    let mut ip_table = Table::new(IpVersion::Ipv4);

    //Add a prerouting rule to redirect all traffic to the client application
    let new_rule = table
        .Chain(Chain::Prerouting)
        .create_rule()
        .destination(client_ip)
        .target(format!("REDIRECT --to-ports {}", client_port))
        .build();

    //Insert the rule into the iptable chain
    if let Err(err) = table.insert_rule(new_rule) {
        eprintln!("Failed to insert iptables rule: {}", err);
        return;
    }

    println!("All traffic was redirected to {}:{}", client_ip, client_port);
}