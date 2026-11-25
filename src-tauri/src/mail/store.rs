//! The mails in the inbox of all accounts, read or not – single source for tray and window.
//!
//! The inbox is a transit station: a mail stays listed until it leaves the inbox (done, task,
//! filed, deleted). Being read only changes how it looks.

use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::watch;

use super::model::MailSummary;

#[derive(Debug, Clone, Default, Serialize)]
pub struct AccountMails {
    pub account_id: String,
    pub account: String,
    /// All unread mails in the inbox on the server.
    pub unread_total: usize,
    /// All mails in the inbox on the server; `mails` holds at most the newest ones.
    pub inbox_total: usize,
    /// The newest mails in the inbox, read or not, newest first.
    pub mails: Vec<MailSummary>,
    /// Pinned (flagged) mails, read or not, newest first.
    pub pinned: Vec<MailSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AccountInfo {
    pub account_id: String,
    pub account: String,
    pub unread_total: usize,
    pub inbox_total: usize,
}

/// What the window shows: per-account counts and all mails, newest first.
#[derive(Debug, Clone, Serialize)]
pub struct Snapshot {
    pub accounts: Vec<AccountInfo>,
    pub mails: Vec<MailSummary>,
    pub pinned: Vec<MailSummary>,
}

pub struct MailStore {
    accounts: Mutex<HashMap<String, AccountMails>>,
    /// Bumped on every change; tray and window listen to it.
    version: watch::Sender<u64>,
}

impl Default for MailStore {
    fn default() -> Self {
        Self { accounts: Mutex::default(), version: watch::channel(0).0 }
    }
}

impl MailStore {
    pub fn subscribe(&self) -> watch::Receiver<u64> {
        self.version.subscribe()
    }

    fn changed(&self) {
        self.version.send_modify(|v| *v += 1);
    }

    /// Replaces the inbox list of an account after a mailbox check.
    pub fn set_account(&self, account_id: &str, account: &str, unread_total: usize, inbox_total: usize, mut mails: Vec<MailSummary>, mut pinned: Vec<MailSummary>) {
        mails.sort_by_key(|m| std::cmp::Reverse(m.uid));
        pinned.sort_by_key(|m| std::cmp::Reverse(m.uid));
        let entry = AccountMails { account_id: account_id.into(), account: account.into(), unread_total, inbox_total, mails, pinned };
        self.accounts.lock().unwrap().insert(account_id.into(), entry);
        self.changed();
    }

    /// Mails already known for an account, by UID – kept across checks with their findings.
    pub fn known(&self, account_id: &str) -> HashMap<u32, MailSummary> {
        self.accounts
            .lock()
            .unwrap()
            .get(account_id)
            .map(|a| a.mails.iter().chain(a.pinned.iter()).map(|m| (m.uid, m.clone())).collect())
            .unwrap_or_default()
    }

    /// Removes a mail that left the inbox (archived, task, filed, deleted) right away – the next
    /// check confirms it. Also from the pinned ones: they live in the inbox too.
    pub fn remove(&self, account_id: &str, uid: u32) {
        let mut accounts = self.accounts.lock().unwrap();
        let Some(acc) = accounts.get_mut(account_id) else { return };
        let listed = acc.mails.iter().chain(acc.pinned.iter()).find(|m| m.uid == uid).map(|m| m.seen);
        let Some(seen) = listed else { return };
        acc.mails.retain(|m| m.uid != uid);
        acc.pinned.retain(|m| m.uid != uid);
        acc.inbox_total = acc.inbox_total.saturating_sub(1);
        if !seen {
            acc.unread_total = acc.unread_total.saturating_sub(1);
        }
        drop(accounts);
        self.changed();
    }

    /// Marks a mail read or unread right away; it stays in the list.
    pub fn set_seen(&self, account_id: &str, uid: u32, seen: bool) {
        let mut accounts = self.accounts.lock().unwrap();
        let Some(acc) = accounts.get_mut(account_id) else { return };
        // A pinned mail may be listed twice; it counts once.
        let mut changed = false;
        for m in acc.mails.iter_mut().chain(acc.pinned.iter_mut()).filter(|m| m.uid == uid && m.seen != seen) {
            m.seen = seen;
            changed = true;
        }
        if changed {
            acc.unread_total = if seen { acc.unread_total.saturating_sub(1) } else { acc.unread_total + 1 };
        }
        drop(accounts);
        if changed {
            self.changed();
        }
    }

    /// A sender was marked as person or bulk: re-flag the mails already shown.
    pub fn reclassify(&self, addr: &str, bulk: bool) {
        let mut accounts = self.accounts.lock().unwrap();
        let mut changed = false;
        for m in accounts.values_mut().flat_map(|a| a.mails.iter_mut().chain(a.pinned.iter_mut())) {
            if m.from_addr.as_deref().is_some_and(|a| a.eq_ignore_ascii_case(addr)) && m.bulk != bulk {
                m.bulk = bulk;
                changed = true;
            }
        }
        drop(accounts);
        if changed {
            self.changed();
        }
    }

    /// Notify listeners without a data change (e.g. the tray counting mode changed).
    pub fn touch(&self) {
        self.changed();
    }

    /// Unread mails from people among the listed ones (bulk mail excluded).
    pub fn unread_people(&self) -> usize {
        self.accounts.lock().unwrap().values().flat_map(|a| a.mails.iter()).filter(|m| !m.bulk && !m.seen).count()
    }

    pub fn clear(&self) {
        self.accounts.lock().unwrap().clear();
        self.changed();
    }

    pub fn find(&self, id: &str) -> Option<MailSummary> {
        self.accounts.lock().unwrap().values().flat_map(|a| a.mails.iter().chain(a.pinned.iter())).find(|m| m.id == id).cloned()
    }

    pub fn unread_total(&self) -> usize {
        self.accounts.lock().unwrap().values().map(|a| a.unread_total).sum()
    }

    pub fn snapshot(&self) -> Snapshot {
        let accounts = self.accounts.lock().unwrap();
        let mut infos: Vec<AccountInfo> = accounts
            .values()
            .map(|a| AccountInfo { account_id: a.account_id.clone(), account: a.account.clone(), unread_total: a.unread_total, inbox_total: a.inbox_total })
            .collect();
        infos.sort_by(|a, b| a.account.cmp(&b.account));
        let mut mails: Vec<MailSummary> = accounts.values().flat_map(|a| a.mails.iter().cloned()).collect();
        mails.sort_by_key(|m| std::cmp::Reverse(m.date.unwrap_or(0)));
        let mut pinned: Vec<MailSummary> = accounts.values().flat_map(|a| a.pinned.iter().cloned()).collect();
        pinned.sort_by_key(|m| std::cmp::Reverse(m.date.unwrap_or(0)));
        Snapshot { accounts: infos, mails, pinned }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mail(account: &str, uid: u32, date: i64) -> MailSummary {
        MailSummary {
            id: format!("{account}:1:{uid}"),
            account_id: account.into(),
            account: account.into(),
            uid,
            from_name: None,
            from_addr: None,
            subject: format!("Mail {uid}"),
            date: Some(date),
            preview: String::new(),
            has_attachments: false,
            seen: false,
            flagged: false,
            message_id: None,
            bulk: false,
            findings: vec![],
        }
    }

    #[test]
    fn test_counts_snapshot_and_remove() {
        let store = MailStore::default();
        let mut rx = store.subscribe();
        store.set_account("a", "Arbeit", 5, 8, vec![mail("a", 1, 100), mail("a", 3, 300)], vec![]);
        store.set_account("p", "Privat", 1, 2, vec![mail("p", 9, 200)], vec![mail("p", 4, 50)]);
        assert_eq!(store.snapshot().pinned.len(), 1);
        assert!(store.find("p:1:4").is_some());
        assert!(rx.has_changed().unwrap());
        rx.mark_unchanged();

        assert_eq!(store.unread_total(), 6);
        let snap = store.snapshot();
        assert_eq!(snap.mails.iter().map(|m| m.uid).collect::<Vec<_>>(), vec![3, 9, 1]);
        assert_eq!(snap.accounts[0].account, "Arbeit");
        assert_eq!(snap.accounts[0].inbox_total, 8);

        store.remove("a", 3);
        assert_eq!(store.unread_total(), 5);
        assert_eq!(store.snapshot().accounts[0].inbox_total, 7);
        assert!(store.find("a:1:3").is_none());
        assert!(rx.has_changed().unwrap());
        rx.mark_unchanged();

        // Unknown UID: no change, no event.
        store.remove("a", 42);
        assert!(!rx.has_changed().unwrap());
        assert_eq!(store.known("a").len(), 1);

        // A pinned mail that leaves the inbox leaves the pinned list too.
        store.remove("p", 4);
        assert!(store.snapshot().pinned.is_empty());
    }

    #[test]
    fn test_read_mails_stay_listed() {
        let store = MailStore::default();
        store.set_account("a", "Arbeit", 2, 2, vec![mail("a", 1, 100), mail("a", 2, 200)], vec![]);
        assert_eq!(store.unread_people(), 2);

        store.set_seen("a", 1, true);
        let snap = store.snapshot();
        assert_eq!(snap.mails.len(), 2, "read, but still in the inbox");
        assert!(snap.mails.iter().find(|m| m.uid == 1).unwrap().seen);
        assert_eq!(store.unread_total(), 1);
        assert_eq!(store.unread_people(), 1);

        // Twice is no change; back to unread counts again.
        store.set_seen("a", 1, true);
        assert_eq!(store.unread_total(), 1);
        store.set_seen("a", 1, false);
        assert_eq!(store.unread_total(), 2);

        // Leaving the inbox as a read mail does not touch the unread count.
        store.set_seen("a", 2, true);
        store.remove("a", 2);
        assert_eq!(store.unread_total(), 1);
        assert_eq!(store.snapshot().accounts[0].inbox_total, 1);
    }
}
