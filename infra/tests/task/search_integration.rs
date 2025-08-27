#[cfg(test)]
mod tests {
    use domain::{TaskRepository, TaskBuilder, TaskStatus};
    use domain::task::repository::{SearchOptions, SortBy, SortOrder};
    use domain::shared_kernel::traits::searchable::SearchCriteria;
    use domain::InMemoryTaskRepository;

    #[tokio::test]
    async fn test_search_tasks_by_query() {
        let task1 = TaskBuilder::with_name_and_sessions("Write documentation".to_string(), 4)
            .with_description("Create user guide for the new feature".to_string())
            .with_tags(vec!["docs".to_string(), "urgent".to_string()])
            .build()
            .unwrap();
        
        let task2 = TaskBuilder::with_name_and_sessions("Fix bugs".to_string(), 2)
            .with_description("Fix critical issues in production".to_string())
            .with_tags(vec!["bug".to_string(), "urgent".to_string()])
            .build()
            .unwrap();
        
        let task3 = TaskBuilder::with_name_and_sessions("Design UI".to_string(), 3)
            .with_description("Create mockups for new dashboard".to_string())
            .with_tags(vec!["design".to_string()])
            .build()
            .unwrap();
        
        let repo = InMemoryTaskRepository::new();
        repo.create(task1).await.unwrap();
        repo.create(task2).await.unwrap();
        repo.create(task3).await.unwrap();
        
        let options = SearchOptions {
            criteria: SearchCriteria::new().with_query("documentation".to_string()),
            sort_by: None,
            sort_order: None,
        };
        
        let results = repo.search(options).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Write documentation");
        
        let fuzzy_results = repo.search_fuzzy("fix prod").await.unwrap();
        assert_eq!(fuzzy_results.len(), 1);
        assert_eq!(fuzzy_results[0].name, "Fix bugs");
    }
    
    #[tokio::test]
    async fn test_search_with_filters() {
        let mut task1 = TaskBuilder::with_name_and_sessions("Task 1".to_string(), 1)
            .with_tags(vec!["work".to_string()])
            .build()
            .unwrap();
        task1.increment_session().unwrap();
        
        let task2 = TaskBuilder::with_name_and_sessions("Task 2".to_string(), 4)
            .with_tags(vec!["personal".to_string()])
            .build()
            .unwrap();
        
        let mut task3 = TaskBuilder::with_name_and_sessions("Task 3".to_string(), 2)
            .with_tags(vec!["work".to_string()])
            .build()
            .unwrap();
        task3.pause().unwrap();
        
        let repo = InMemoryTaskRepository::new();
        repo.create(task1).await.unwrap();
        repo.create(task2).await.unwrap();
        repo.create(task3).await.unwrap();
        
        let completed_tasks = repo.get_by_status(TaskStatus::Completed).await.unwrap();
        assert_eq!(completed_tasks.len(), 1);
        
        let work_tasks = repo.get_by_tags(&["work".to_string()]).await.unwrap();
        assert_eq!(work_tasks.len(), 2);
        
        let options = SearchOptions {
            criteria: SearchCriteria::new()
                .with_tags(vec!["work".to_string()])
                .with_status("paused".to_string()),
            sort_by: Some(SortBy::Name),
            sort_order: Some(SortOrder::Ascending),
        };
        
        let filtered = repo.search(options).await.unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Task 3");
    }
    
    #[tokio::test]
    async fn test_search_with_sorting() {
        let task1 = TaskBuilder::with_name_and_sessions("Beta Task".to_string(), 3)
            .build()
            .unwrap();
        
        let mut task2 = TaskBuilder::with_name_and_sessions("Alpha Task".to_string(), 4)
            .build()
            .unwrap();
        task2.increment_session().unwrap();
        task2.increment_session().unwrap();
        
        let task3 = TaskBuilder::with_name_and_sessions("Gamma Task".to_string(), 2)
            .build()
            .unwrap();
        
        let repo = InMemoryTaskRepository::new();
        repo.create(task1).await.unwrap();
        repo.create(task2).await.unwrap();
        repo.create(task3).await.unwrap();
        
        let options = SearchOptions {
            criteria: SearchCriteria::new(),
            sort_by: Some(SortBy::Name),
            sort_order: Some(SortOrder::Ascending),
        };
        
        let sorted = repo.search(options).await.unwrap();
        assert_eq!(sorted[0].name, "Alpha Task");
        assert_eq!(sorted[1].name, "Beta Task");
        assert_eq!(sorted[2].name, "Gamma Task");
        
        let options = SearchOptions {
            criteria: SearchCriteria::new(),
            sort_by: Some(SortBy::SessionsCompleted),
            sort_order: Some(SortOrder::Descending),
        };
        
        let sorted = repo.search(options).await.unwrap();
        assert_eq!(sorted[0].name, "Alpha Task");
        assert_eq!(sorted[0].current_sessions, 2);
    }
}